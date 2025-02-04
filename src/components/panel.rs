#![allow(unused_variables)]
use crate::app;

pub fn top_panel(app: &mut app::App, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
        egui::widgets::global_theme_preference_switch(ui);
        // NOTE: no File->Quit on web pages!
        if cfg!(not(target_arch = "wasm32")) {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        }
        ui.menu_button("Settings", |ui| {
            ui.checkbox(&mut app.logging_window, "Toggle log window");
            if cfg!(debug_assertions) {
                ui.checkbox(&mut app.memory_window, "Toggle memory window");
            }
        });
    });
}

pub fn bottom_panel(app: &mut app::App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.scope(|ui| {
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
        egui::warn_if_debug_build(ui);
    });
}
