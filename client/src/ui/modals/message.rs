use crate::ui::modals::Modal;
use egui::{Id, WidgetText};

pub struct MessageModal {
    id: i64,
    name: String,
    message: WidgetText,

    width: f32,

    is_open: bool,
}

impl Default for MessageModal {
    fn default() -> Self {
        Self {
            id: rand::random(),
            name: "Modal".to_string(),
            message: WidgetText::default(),

            is_open: true,

            width: 100.0,
        }
    }
}

impl Modal for MessageModal {
    fn show(&mut self, ui: &egui::Ui) {
        if self.is_open {
            let modal = egui::Modal::new(Id::new(self.id)).show(ui.ctx(), |ui| {
                ui.set_width(self.width);

                ui.vertical_centered_justified(|ui| {
                    ui.heading(&self.name);
                });

                ui.add_space(16.0);

                ui.label(self.message.clone());

                ui.add_space(16.0);

                ui.vertical_centered_justified(|ui| {
                    if ui.button("Close").clicked() {
                        self.close()
                    }
                });
            });

            if modal.should_close() {
                self.close()
            }
        }
    }

    fn is_closed(&self) -> bool {
        !self.is_open
    }
}

impl MessageModal {
    pub fn error(message: &str) -> Self {
        MessageModal::default()
            .with_message(message)
            .with_name("Error ❎")
            .with_width(300.0)
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_message(mut self, message: impl Into<WidgetText>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    fn close(&mut self) {
        self.is_open = false;
    }
}
