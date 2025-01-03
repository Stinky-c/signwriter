use egui::{Response, Widget};
use log::{error, info, warn};
use poll_promise::Promise;
use reqwest::Method;
use std::sync::{Arc, Mutex};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    // Enables the separate logging window
    // #[serde(skip)]
    logging_window: bool,
    edit_json: bool,
    #[serde(skip)]
    http_connection: Arc<reqwest::blocking::Client>,

    #[serde(skip)]
    response: Option<Promise<reqwest::blocking::Response>>,

    #[serde(skip)]
    body: Option<Promise<String>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
            logging_window: false,
            edit_json: false,
            http_connection: reqwest::blocking::Client::new().into(),
            // http_result: None,
            // caddy_body: None,
            response: Default::default(),
            body: Default::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.logging_window == true {
            egui::Window::new("Logs").show(ctx, |ui| egui_logger::logger_ui().show(ui));
        }

        egui::Window::new("HTTP").show(ctx, |ui| {
            if ui.button("Refresh").clicked() {
                self.body = crate::http::try_http::<String>(
                    self.http_connection.get("https://httpbin.org/json"),
                    move |res| res.text().unwrap(),
                );
            }
            ui.checkbox(&mut self.edit_json, "Enable editing");

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
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                ui.menu_button("Settings", |ui| {
                    ui.checkbox(&mut self.logging_window, "Toggle log window");
                });
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Caddyapp");

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
                crate::app::powered_by_egui_and_eframe(ui);
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
