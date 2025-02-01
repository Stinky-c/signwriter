use crate::app;
use crate::models::prelude::{Router, Service};
use log::info;
use std::fmt::format;

pub fn central_pane(app: &mut app::App, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Signwriter");

    ui.horizontal(|ui| {
        ui.label("Write something: ");
        ui.text_edit_singleline(&mut app.label);
    });

    ui.add(egui::Slider::new(&mut app.value, 0.0..=10.0).text("value"));
    if ui.button("Increment").clicked() {
        info!("increment");
        app.value += 1.0;
    }

    ui.add(egui::github_link_file!(
        "https://github.com/emilk/eframe_template/blob/main/",
        "Source code."
    ));
    // Routers
    ui.group(|ui| {
        ui.heading("Create New Router");
        edit_field(ui, "Name", &mut app.new_router.name);
        edit_field(ui, "Service:", &mut app.new_router.service);
        edit_field(ui, "Rule:", &mut app.new_router.rule);
        if ui.button("Create Router").clicked() {
            info!("Creating new router");
            // Replace `app.new_router` with a default, and move original
            let router = std::mem::take(&mut app.new_router);
            app.routers.push(router);
        };

        if !app.routers.is_empty() {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                for value in app.routers.iter() {
                    ui.with_layout(egui::Layout::default(), move |ui| {
                        ui.group(|ui| {
                            ui.label(format!("Route: {}", value.name));
                            ui.label(format!("Rule: {}", value.rule));
                            ui.label(format!("Service: {}", value.service));
                        });
                    });
                }
            });
        };
    });

    // Services
    ui.group(|ui| {
        ui.heading("Create New services");
        edit_field(ui, "Name", &mut app.new_service.service_name);
        // TODO: new list, CRUD ops on list
        edit_list(ui, "URLs", &mut app.new_service.urls);
        if ui.button("Create Service").clicked() {
            info!("Creating new service");
            let service = std::mem::take(&mut app.new_service);
            app.services.push(service);
        }

        for (index, value) in app.services.iter().enumerate() {
            ui.group(|ui| {
                ui.label(format!("Service #{}", index));
                ui.label(format!("Name: {}", value.service_name));
                ui.label(format!("URls: {:?}", value.urls));
            });
        }
    });
}

pub fn edit_field(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, text: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(text);
    });
}

pub fn list_line(ui: &mut egui::Ui, str_list: Vec<String>) {
    for (index, value) in str_list.iter().enumerate() {
        ui.label(format!("{}: {}", index, value));
    }
}

pub fn edit_list(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, str_list: &mut [String]) {
    ui.label(label);
    for (index, value) in str_list.iter_mut().enumerate() {
        edit_field(ui, index.to_string(), value);
    }
}
