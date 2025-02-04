use crate::app;
use crate::widgets::list_container;
use log::info;

pub fn central_pane(app: &mut app::App, ui: &mut egui::Ui) {
    ui.heading("Signwriter");

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

        list_container::ListContainer::new("Urls", &mut app.new_service.urls).show(ui);

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
