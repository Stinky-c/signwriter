use egui::{Context, Id, RichText, Ui, Widget};
use log::debug;

#[derive(Clone, Default)]
pub struct ListContainerState {
    text: String,
}

impl ListContainerState {
    fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_temp(id))
    }
    fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
}

/// A list editor providing a view into the provided items. Enables removing and adding new items
/// TODO: add validator
pub struct ListContainer<'a> {
    id: Option<Id>,
    label: RichText,
    items: &'a mut Vec<String>,
}

impl<'a> ListContainer<'a> {
    pub fn new(label: &'static str, items: &'a mut Vec<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            items,
        }
    }

    pub fn show(self, ui: &mut Ui) {
        let id = match self.id {
            Some(id) => id,
            None => ui.id(),
        };
        // Load state
        let mut state = ListContainerState::load(ui.ctx(), id).unwrap_or_default();

        ui.label(self.label);

        // Show edit line and add button
        ui.horizontal(|ui| {
            let edit_line = egui::TextEdit::singleline(&mut state.text)
                .hint_text("New Item")
                .show(ui);

            let button = egui::Button::new("Add").ui(ui);

            // edit box lost focus and enter was hit
            // OR add button was clicked
            if (edit_line.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || button.clicked()
            {
                let x = std::mem::take(&mut state.text);
                self.items.push(x);
            }
        });

        // handle showing new items, and deleting items
        let mut iter_update: Option<usize> = None;
        ui.vertical(|ui| {
            for (index, item) in self.items.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(item.clone());
                    if ui.button("X").clicked() {
                        iter_update = Some(index);
                        debug!("removing item {}", index);
                    };
                });
            }
        });
        match iter_update {
            None => {}
            Some(index) => {
                self.items.remove(index);
            }
        };

        // Update state
        state.clone().store(ui.ctx(), id);
    }
}

// impl<'a> Widget for ListContainer<'a> {
//     fn ui(self, ui: &mut Ui) -> Response {
//         self.show(ui)
//     }
// }
//
// impl WidgetWithState for ListContainer<'_> {
//     type State = ListContainerState;
// }
