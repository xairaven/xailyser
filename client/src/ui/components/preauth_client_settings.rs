use crate::context::Context;
use crate::ui::tabs::settings_client::SettingsClientTab;
use egui::{CentralPanel, RichText, TopBottomPanel};

pub struct PreAuthClientSettingsComponent {
    tab: SettingsClientTab,
    is_opened: bool,
}

impl PreAuthClientSettingsComponent {
    pub fn new(ctx: &Context) -> Self {
        Self {
            tab: SettingsClientTab::new(ctx),
            is_opened: false,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let theme = ctx.client_settings.theme.into_aesthetix_theme();
        TopBottomPanel::top("TOP_PANEL_PRE_AUTH_CLIENT_SETTINGS")
            .frame(
                egui::Frame::new()
                    .inner_margin(theme.margin_style())
                    .fill(theme.bg_primary_color_visuals()),
            )
            .show(ui.ctx(), |ui| {
                ui.columns(3, |columns| {
                    const MAIN_COLUMN: usize = 1;
                    const RIGHT_COLUMN: usize = 2;

                    columns[MAIN_COLUMN].vertical_centered(|ui| {
                        ui.heading(RichText::new("⚙ Client Settings").size(25.0));
                    });

                    columns[RIGHT_COLUMN].with_layout(
                        egui::Layout::right_to_left(egui::Align::Min),
                        |ui| {
                            if ui.button("✖").clicked() {
                                self.close();
                            }
                        },
                    );
                });
            });

        CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(theme.margin_style())
                    .fill(theme.bg_primary_color_visuals()),
            )
            .show(ui.ctx(), |ui| {
                self.tab.show(ui, ctx);
            });
    }

    // Synchronizing pre-auth and post-auth settings tabs
    pub fn update_tab(&mut self, ctx: &Context) {
        self.tab = SettingsClientTab::new(ctx);
    }

    pub fn is_opened(&self) -> bool {
        self.is_opened
    }

    pub fn open(&mut self) {
        self.is_opened = true;
    }

    pub fn close(&mut self) {
        self.is_opened = false;
    }
}
