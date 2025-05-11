use crate::context::Context;
use crate::ui;
use crate::ui::styles;
use crate::ui::tabs::Tab;
use crate::ui::tabs::about::AboutTab;
use crate::ui::tabs::inspector::InspectorTab;
use crate::ui::tabs::settings_client::SettingsClientTab;
use crate::ui::tabs::settings_server::SettingsServerTab;
use crate::ui::tabs::stats::StatsTab;
use crate::ui::tabs::status::StatusTab;
use crate::ws::request::UiClientRequest;
use egui::{CentralPanel, RichText, SidePanel};
use std::collections::BTreeMap;

pub const MENU_PANEL_MIN_WIDTH: f32 = ui::MIN_WINDOW_WIDTH * 0.25;

pub struct RootComponent {
    active_tab: Tab,
    tabs: BTreeMap<Tab, String>,

    logout_requested: bool,

    pub status_tab: StatusTab,
    pub inspector_tab: InspectorTab,
    pub stats_tab: StatsTab,
    pub settings_client_tab: SettingsClientTab,
    pub settings_server_tab: SettingsServerTab,
    pub about_tab: AboutTab,
}

impl RootComponent {
    pub fn new(ctx: &Context) -> Self {
        Self {
            active_tab: Default::default(),

            tabs: [
                (Tab::Status, Tab::Status.to_string()),
                (Tab::Inspector, Tab::Inspector.to_string()),
                (Tab::Stats, Tab::Stats.to_string()),
                (Tab::ClientSettings, Tab::ClientSettings.to_string()),
                (Tab::ServerSettings, Tab::ServerSettings.to_string()),
                (Tab::About, Tab::About.to_string()),
                (Tab::Logout, Tab::Logout.to_string()),
                (Tab::Exit, Tab::Exit.to_string()),
            ]
            .into_iter()
            .collect(),

            logout_requested: false,

            status_tab: StatusTab::new(ctx),
            inspector_tab: Default::default(),
            stats_tab: Default::default(),
            settings_client_tab: SettingsClientTab::new(ctx),
            settings_server_tab: Default::default(),
            about_tab: Default::default(),
        }
    }
}

impl RootComponent {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let theme = ctx.client_settings.theme.into_aesthetix_theme();

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
                        ui.heading(styles::heading::huge(&t!(
                            "Component.Root.Dashboard"
                        )));
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

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{}: ", t!("Text.LastUpdate")))
                                .size(styles::text::SMALL)
                                .color(styles::colors::SILENT),
                        );
                        match &ctx.heartbeat.last_sync {
                            None => {
                                ui.label(
                                    RichText::new(t!("Text.LastUpdate.Never"))
                                        .size(styles::text::SMALL)
                                        .color(styles::colors::OUTDATED_DARK),
                                );
                            },
                            Some(last_sync) => {
                                let mut text = RichText::new(
                                    last_sync.format(styles::TIME_FORMAT).to_string(),
                                )
                                .size(styles::text::SMALL);

                                if ctx.heartbeat.is_timeout(&ctx.client_settings) {
                                    text = text.color(styles::colors::OUTDATED_DARK);
                                } else {
                                    text = text.color(styles::colors::UPDATED_DARK);
                                }
                                ui.label(text);
                            },
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("🔃").size(styles::text::SMALL),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            ctx.heartbeat.try_ping(&ctx.ui_client_requests_tx);
                        }
                    });
                });
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
                    self.status_tab.show(ui, ctx);
                },
                Tab::Inspector => {
                    self.inspector_tab.show(ui, ctx);
                },
                Tab::Stats => {
                    self.stats_tab.show(ui, ctx);
                },
                Tab::ClientSettings => {
                    self.settings_client_tab.show(ui, ctx);
                },
                Tab::ServerSettings => {
                    self.settings_server_tab.show(ui, ctx);

                    if self.settings_server_tab.reboot_requested {
                        self.settings_server_tab.reboot_requested = false;
                        self.logout_requested = true;
                        self.active_tab = Tab::Status;
                    }
                },
                Tab::About => {
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

    pub fn logout_requested(&self) -> bool {
        self.logout_requested
    }

    pub fn logout(&mut self, ctx: &Context) {
        let _ = ctx
            .ui_client_requests_tx
            .try_send(UiClientRequest::CloseConnection);
        self.logout_requested = false;
        self.update_client_settings_info(ctx);
        log::info!("Logged out!");
    }

    // Synchronizing pre-auth and post-auth settings tabs
    pub fn update_client_settings_info(&mut self, ctx: &Context) {
        self.settings_client_tab = SettingsClientTab::new(ctx);
    }
}
