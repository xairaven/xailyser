use crate::ui::components::settings::SettingsComponent;
use crate::ui::components::status::StatusComponent;
use crate::ui::components::Tab;
use egui::{CentralPanel, SidePanel};
use std::thread::JoinHandle;
use strum::IntoEnumIterator;

#[derive(Default)]
pub struct UiRoot {
    pub net_thread: Option<JoinHandle<()>>,

    tab_current: Tab,

    status_component: StatusComponent,
    settings_component: SettingsComponent,
}

impl UiRoot {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        SidePanel::left("MENU_PANEL")
            .resizable(false)
            .min_width(ui.available_width() * 0.25)
            .show_separator_line(true)
            .show_inside(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.heading("Menu");
                });

                ui.add_space(10.0);

                ui.vertical_centered_justified(|ui| {
                    for tab in Tab::iter() {
                        if ui.button(format!("{}", tab)).clicked() {
                            self.tab_current = tab;
                        }
                        ui.add_space(2.0);
                    }
                });
            });

        CentralPanel::default().show_inside(ui, |ui| match self.tab_current {
            Tab::Status => {
                self.status_component.show(ui);
            },
            Tab::Settings => {
                self.settings_component.show(ui);
            },
            Tab::Exit => {
                todo!()
            },
        });
    }
}
