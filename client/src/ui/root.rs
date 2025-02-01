use crate::ui::components::menu::Menu;
use egui::{CentralPanel, SidePanel};
use std::thread::JoinHandle;

#[derive(Default)]
pub struct UiRoot {
    pub net_thread: Option<JoinHandle<()>>,

    menu: Menu,
}

impl UiRoot {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        SidePanel::left("SETTINGS_PANEL")
            .resizable(false)
            .min_width(ui.available_width() * 0.25)
            .show_separator_line(true)
            .show_inside(ui, |ui| {
                self.menu.show(ui);
            });

        CentralPanel::default().show_inside(ui, |ui| {
            ui.label("Root. Dashboard");
        });
    }
}
