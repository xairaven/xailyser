use crate::context::Context;
use crate::ui::windows::Window;
use egui::{Id, WidgetText};
use std::sync::{Arc, Mutex};

pub struct MessageWindow {
    id: i64,
    name: String,
    message: WidgetText,

    width: f32,
    height: f32,

    collapsible: bool,

    is_open: bool,
}

impl Default for MessageWindow {
    fn default() -> Self {
        Self {
            id: rand::random(),
            name: "Window".to_string(),
            message: WidgetText::default(),

            is_open: true,

            collapsible: true,

            width: 100.0,
            height: 100.0,
        }
    }
}

impl Window for MessageWindow {
    fn show(&mut self, ui: &egui::Ui, _ctx: Arc<Mutex<Context>>) {
        egui::Window::new(&self.name)
            .id(Id::new(self.id))
            .open(&mut self.is_open)
            .min_width(self.width)
            .min_height(self.height)
            .collapsible(self.collapsible)
            .show(ui.ctx(), |ui| {
                ui.label(self.message.clone());
            });
    }

    fn is_closed(&self) -> bool {
        !self.is_open
    }
}

impl MessageWindow {
    pub fn error(message: &str) -> Self {
        MessageWindow::default()
            .with_message(message)
            .with_name("Error ❎")
            .with_height(500.0)
            .with_width(300.0)
            .with_collapsible(false)
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

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn with_collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
}
