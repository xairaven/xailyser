use crate::communication::request::UiClientRequest;
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

    logout_requested: bool,

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
                (Tab::Logout, Tab::Logout.to_string()),
                (Tab::Exit, Tab::Exit.to_string()),
            ]
            .into_iter()
            .collect(),

            logout_requested: false,

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
            .show(ui.ctx(), |ui| match self.active_tab {
                Tab::Status => {
                    self.tab_heading(ui);
                    self.status_tab.show(ui, ctx);
                },
                Tab::Settings => {
                    self.tab_heading(ui);
                    self.settings_tab.show(ui, ctx);
                },
                Tab::About => {
                    self.tab_heading(ui);
                    self.about_tab.show(ui, ctx);
                },
                Tab::Logout => {
                    self.logout_requested = true;
                    self.active_tab = Tab::Status;
                },
                Tab::Exit => {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                },
            });
    }

    fn tab_heading(&self, ui: &mut egui::Ui) {
        ui.add_space(13.0);
        ui.heading(
            egui::RichText::new(
                self.tabs
                    .get(&self.active_tab)
                    .unwrap_or(&String::from("Tab")),
            )
            .size(25.0),
        );
    }

    pub fn logout_requested(&self) -> bool {
        self.logout_requested
    }

    pub fn logout(&mut self, ctx: &Context) {
        let _ = ctx
            .ui_client_requests_tx
            .try_send(UiClientRequest::CloseConnection);
        self.logout_requested = false;
        log::info!("Logged out!");
    }
}
