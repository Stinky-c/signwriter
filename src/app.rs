use crate::client::*;
use crate::components;
use crate::models::prelude::{Router, Service};
use crate::thread;
use eyre::{eyre, Result};
use log::{error, info};
use poll_promise::Promise;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    pub(crate) label: String,
    #[serde(skip)] // This how you opt out of serialization of a field
    pub(crate) value: f32,

    /// Enables the separate logging window
    pub(crate) logging_window: bool,
    pub(crate) memory_window: bool,
    grpc_addr: String,
    edit_json: bool,

    #[serde(skip)]
    grpc_promise: Option<Promise<Result<()>>>,
    #[serde(skip)]
    client: Arc<RwLock<Option<EtcdClient>>>,

    #[serde(skip)]
    body: Option<Promise<String>>,
    #[serde(skip)]
    pub(crate) routers: Vec<Router>,
    #[serde(skip)]
    pub(crate) services: Vec<Service>,

    #[serde(skip)]
    pub(crate) new_service: Service,
    #[serde(skip)]
    pub(crate) new_router: Router,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            logging_window: false,
            memory_window: false,
            edit_json: false,
            grpc_promise: None,
            body: Default::default(),
            grpc_addr: "http://localhost:2379".to_string(),
            client: Arc::new(RwLock::new(None)),
            services: Vec::new(),
            new_service: Service {
                service_name: "abc".to_string(),
                urls: vec![
                    "http://localhost:2378".to_string(),
                    "http://localhost:2379".to_string(),
                ],
            },
            routers: Vec::new(),
            new_router: Router::default(),
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

        if cfg!(debug_assertions) && self.memory_window {
            egui::Window::new("Memory").show(ctx, |ui| ctx.memory_ui(ui));
        }

        egui::Window::new("gRPC Connection").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Address");
                ui.text_edit_singleline(&mut self.grpc_addr);
            });
            if ui.button("Connect").clicked() {
                // TODO: make a wrapper around this. I will likely only use promises to show spinners
                let (sender, promise) = Promise::new();
                let c2 = self.client.clone();
                let addr = self.grpc_addr.clone(); // Clone addr to move into thread
                thread::spawn_thread(async move {
                    let client = match EtcdClient::new(addr.clone()) {
                        Ok(c) => {
                            info!("Connected at: {}", addr);
                            c
                        }
                        Err(e) => {
                            let msg = format!("Failed to connect: {}", e);
                            error!("{}", msg);
                            sender.send(Err(eyre!(msg)));
                            return;
                        }
                    };
                    match c2.write() {
                        Ok(mut c) => {
                            c.replace(client);
                            sender.send(Ok(()));
                        }
                        Err(e) => {
                            let msg = format!("Lock is poisoned. Please open a issue. {}", e);
                            error!("{}", msg);
                            sender.send(Err(eyre!(msg)));
                        }
                    }
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

        egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| components::panel::top_panel(self, ui));
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            components::panel::bottom_panel(self, ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            components::pane::central_pane(self, ui);
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
