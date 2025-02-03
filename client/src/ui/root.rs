use crate::context::Context;
use crate::ui;
use crate::ui::components::settings::SettingsComponent;
use crate::ui::components::status::StatusComponent;
use crate::ui::menu::{Menu, Tab};
use egui::{CentralPanel, SidePanel};
use std::thread::JoinHandle;

pub const MENU_PANEL_MIN_WIDTH: f32 = ui::MIN_WINDOW_WIDTH * 0.25;

#[derive(Default)]
pub struct UiRoot {
    pub net_thread: Option<JoinHandle<()>>,

    menu: Menu,

    status_component: StatusComponent,
    settings_component: SettingsComponent,
}

impl UiRoot {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        SidePanel::left("MENU_PANEL")
            .resizable(false)
            .min_width(MENU_PANEL_MIN_WIDTH)
            .show_separator_line(true)
            .show_inside(ui, |ui| {
                self.menu.show(ui, ctx);
            });

        CentralPanel::default().show_inside(ui, |ui| match self.menu.tab_current {
            Tab::Status => {
                self.status_component.show(ui, ctx);
            },
            Tab::Settings => {
                self.settings_component.show(ui, ctx);
            },
            Tab::About => {
                todo!()
            },
            Tab::Exit => {
                todo!()
            },
        });
    }
}
