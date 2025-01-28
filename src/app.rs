use crate::client::*;
use crate::thread;
use eyre::{eyre, Result};
use log::{error, info};
use poll_promise::Promise;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,
    #[serde(skip)] // This how you opt out of serialization of a field
    value: f32,

    /// Enables the separate logging window
    logging_window: bool,
    grpc_addr: String,
    edit_json: bool,

    #[serde(skip)]
    grpc_promise: Option<Promise<Result<()>>>,
    #[serde(skip)]
    client: Arc<Mutex<Option<EtcdClient>>>,

    #[serde(skip)]
    body: Option<Promise<String>>,
    // body: Option<Receiver<String>>,
    #[serde(skip)]
    isweb: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            logging_window: false,
            edit_json: false,
            grpc_promise: None,
            body: Default::default(),
            isweb: cfg!(target_arch = "wasm32"),
            grpc_addr: "http://localhost:2379".to_string(),
            client: Arc::new(Mutex::new(None)),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        if let Some(storage) = cc.storage {
            // Linux: /home/UserName/.local/share/APP_ID
            // macOS: /Users/UserName/Library/Application Support/APP_ID
            // Windows: C:\Users\UserName\AppData\Roaming\APP_ID\data
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.logging_window {
            egui::Window::new("Log").show(ctx, |ui| egui_logger::logger_ui().show(ui));
        }

        egui::Window::new("gRPC Connection").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Address");
                ui.text_edit_singleline(&mut self.grpc_addr);
            });
            if ui.button("Connect").clicked() {
                // TODO: make a wrapper around this. I will likely only use promises to show spinners
                let (sender, promise) = Promise::new();
                let client_lock = self.client.clone(); // Grab a ref to the client lock
                let addr = self.grpc_addr.clone(); // Clone addr to move into thread
                thread::spawn_thread(async move {
                    let client = match EtcdClient::new(addr.clone()) {
                        Ok(v) => v,
                        Err(e) => {
                            let s = format!("Failed to connect: {}", e);
                            error!("{}", s);
                            sender.send(Err(eyre!(s)));
                            return;
                        }
                    };

                    let mut lock = match client_lock.lock() {
                        Ok(lock) => lock,
                        Err(_) => {
                            error!("Failed to acquire client lock");
                            sender.send(Err(eyre!("Failed to acquire lock")));
                            return;
                        }
                    };
                    lock.replace(client);
                    info!("Connected at: {}", addr);
                    sender.send(Ok(()));
                });
                self.grpc_promise = Some(promise);
            }

            if let Some(promise) = &self.grpc_promise {
                if let Some(v) = promise.ready() {
                    match v {
                        Ok(_) => {
                            ui.label("Connected!");
                        }
                        Err(e) => {
                            ui.label("Failed to connect");
                            ui.label(format!("{}", e));
                        }
                    }
                } else {
                    // Connect dispatched
                    ui.spinner();
                }
            }
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                // NOTE: no File->Quit on web pages!
                if !self.isweb {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }
                ui.menu_button("Settings", |ui| {
                    ui.checkbox(&mut self.logging_window, "Toggle log window");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Signwriter");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                info!("increment");
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
}

pub(crate) fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
