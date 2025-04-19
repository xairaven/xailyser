use crate::context::Context;
use crate::ui::modals::{Modal, ModalFields};
use egui::{Ui, WidgetText};

#[derive(Default)]
pub struct MessageModal {
    modal_fields: ModalFields,
    message: WidgetText,
}

impl Modal for MessageModal {
    fn show_content(&mut self, ui: &mut Ui, _ctx: &mut Context) {
        ui.label(self.message.clone());

        ui.add_space(16.0);

        ui.vertical_centered_justified(|ui| {
            if ui.button("Close").clicked() {
                self.close()
            }
        });
    }

    fn close(&mut self) {
        self.modal_fields.is_open = false;
    }

    fn modal_fields(&self) -> &ModalFields {
        &self.modal_fields
    }
}

impl MessageModal {
    pub fn error(message: &str) -> Self {
        MessageModal::default()
            .with_message(message)
            .with_title("❎ Error".to_string())
            .with_width(300.0)
    }

    pub fn info(message: &str) -> Self {
        MessageModal::default()
            .with_message(message)
            .with_title("ℹ Info".to_string())
            .with_width(300.0)
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.modal_fields.title = title;
        self
    }

    pub fn with_message(mut self, message: impl Into<WidgetText>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.modal_fields.width = width;
        self
    }
}
