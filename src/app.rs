use crate::proto::*;
use eyre::Result;
use log::info;
use poll_promise::Promise;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,
    #[serde(skip)] // This how you opt out of serialization of a field
    value: f32,

    // Enables the separate logging window
    // #[serde(skip)]
    logging_window: bool,
    edit_json: bool,
    #[serde(skip)]
    http_connection: Option<Arc<reqwest::Client>>,
    #[serde(skip)]
    grpc_connection: Option<Arc<EtcdClient>>,
    #[serde(skip)]
    grpc_promise: Option<Promise<Result<()>>>,
    #[serde(skip)]
    client: Arc<Mutex<Option<EtcdClient>>>,

    #[serde(skip)]
    body: Option<Promise<String>>,
    // body: Option<Receiver<String>>,
    #[serde(skip)]
    isweb: bool,
    // #[serde(skip)]
    // container: Container,
    #[serde(skip)]
    grpc_addr: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            logging_window: false,
            edit_json: false,
            http_connection: Some(Arc::new(reqwest::Client::new())),
            grpc_connection: None,
            grpc_promise: None,
            // http_result: None,
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
        if !self.isweb && self.logging_window {
            egui::Window::new("Logs").show(ctx, |ui| egui_logger::logger_ui().show(ui));
        }

        egui::Window::new("Grpc connection").show(ctx, |ui| {
            ui.label(&self.grpc_addr);
            if ui.button("Connect").clicked() {
                let (sender, promise) = Promise::new();

                // clone Arc, this is grabbing a reference
                let client_lock = self.client.clone();
                let addr = self.grpc_addr.clone();
                tokio::task::spawn(async move {
                    let mut client = EtcdClient::new(addr.clone());
                    client.connect().await.expect("TODO: panic message");
                    client_lock.lock().unwrap().replace(client);
                    info!("Connected to {}", addr);
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
                        Err(_) => {
                            ui.label("Failed to connect");
                        }
                    }
                } else {
                    // Connect dispatched
                    ui.spinner();
                }
            }
        });

        egui::Window::new("HTTP").show(ctx, |ui| {
            if ui.button("Refresh").clicked() {
                let request = self
                    .http_connection
                    .as_ref()
                    .unwrap()
                    .get("https://httpbin.org/json");
                // self.body = crate::http::try_http::<String>(request, move |res| async {
                //     res.text().await.unwrap()
                // });

                let (sender, promise) = Promise::new();
                reqwest_cross::fetch(request, move |x| async {
                    let v = x.unwrap().text().await.unwrap();
                    sender.send(v)
                });
                self.body = Some(promise);
            }
            // ui.checkbox(&mut self.edit_json, "Enable editing");

            // Clean this somehow, and make sure `ui.code_editor` can borrow mutably (might need to break into a different var)
            if let Some(promise) = &self.body {
                if let Some(v) = promise.ready() {
                    ui.add_enabled_ui(self.edit_json, |ui| ui.code_editor(&mut v.clone()));
                } else {
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
                    if !self.isweb {
                        ui.checkbox(&mut self.logging_window, "Toggle log window");
                    };
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
