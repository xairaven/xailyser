use crate::context::Context;
use crate::ui;
use crate::ui::tabs::about::AboutTab;
use crate::ui::tabs::settings::SettingsTab;
use crate::ui::tabs::status::StatusTab;
use crate::ui::tabs::Tab;
use egui::{CentralPanel, SidePanel};
use std::collections::BTreeMap;

pub const MENU_PANEL_MIN_WIDTH: f32 = ui::MIN_WINDOW_WIDTH * 0.25;

pub struct RootComponent {
    active_tab: Tab,
    tabs: BTreeMap<Tab, String>,

    status_tab: StatusTab,
    settings_tab: SettingsTab,
    about_tab: AboutTab,
}

impl Default for RootComponent {
    fn default() -> Self {
        Self {
            active_tab: Default::default(),

            tabs: [
                (Tab::Status, Tab::Status.to_string()),
                (Tab::Settings, Tab::Settings.to_string()),
                (Tab::About, Tab::About.to_string()),
                (Tab::Exit, Tab::Exit.to_string()),
            ]
            .into_iter()
            .collect(),

            status_tab: Default::default(),
            settings_tab: Default::default(),
            about_tab: Default::default(),
        }
    }
}

impl RootComponent {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let theme = ctx.active_theme.into_aesthetix_theme();

        SidePanel::left("MENU_PANEL")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_secondary_color_visuals())
                    .inner_margin(theme.margin_style())
                    .stroke(egui::Stroke::new(1.0, theme.bg_secondary_color_visuals())),
            )
            .min_width(MENU_PANEL_MIN_WIDTH)
            .show_separator_line(true)
            .show(ui.ctx(), |ui| {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.add_space(15.0);
                        ui.heading(egui::RichText::new("Dashboard").size(25.0).strong());
                        egui::warn_if_debug_build(ui);
                    },
                );

                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Min),
                    |ui| {
                        for (tab, label) in &self.tabs {
                            ui.selectable_value(&mut self.active_tab, *tab, label);
                        }
                    },
                );
            });

        // This builds the main central panel that holds the content of the active tab
        CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(theme.margin_style())
                    .fill(theme.bg_primary_color_visuals()),
            )
            .show(ui.ctx(), |ui| {
                ui.add_space(13.0);
                ui.heading(
                    egui::RichText::new(
                        self.tabs
                            .get(&self.active_tab)
                            .unwrap_or(&String::from("Tab")),
                    )
                    .size(25.0),
                );

                match self.active_tab {
                    Tab::Status => {
                        self.status_tab.show(ui, ctx);
                    },
                    Tab::Settings => {
                        self.settings_tab.show(ui, ctx);
                    },
                    Tab::About => {
                        self.about_tab.show(ui, ctx);
                    },
                    Tab::Exit => {
                        todo!()
                    },
                }
            });
    }
}
