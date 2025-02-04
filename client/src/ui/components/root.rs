use crate::context::Context;
use crate::ui;
use crate::ui::components::menu::Menu;
use crate::ui::tabs::settings::SettingsTab;
use crate::ui::tabs::status::StatusTab;
use crate::ui::tabs::Tab;
use egui::{CentralPanel, SidePanel};

pub const MENU_PANEL_MIN_WIDTH: f32 = ui::MIN_WINDOW_WIDTH * 0.25;

#[derive(Default)]
pub struct RootComponent {
    menu: Menu,

    status_tab: StatusTab,
    settings_tab: SettingsTab,
}

impl RootComponent {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        SidePanel::left("MENU_PANEL")
            .resizable(false)
            .min_width(MENU_PANEL_MIN_WIDTH)
            .show_separator_line(true)
            .show_inside(ui, |ui| {
                self.menu.show(ui, ctx);
            });

        CentralPanel::default().show_inside(ui, |ui| match self.menu.active_tab {
            Tab::Status => {
                self.status_tab.show(ui, ctx);
            },
            Tab::Settings => {
                self.settings_tab.show(ui, ctx);
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
