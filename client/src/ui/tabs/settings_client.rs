use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use crate::ui::styles;
use crate::ui::styles::{colors, spacing, themes};
use crate::{config, logging};
use egui::{Checkbox, DragValue, Grid, RichText, TextEdit};
use log::LevelFilter;
use strum::IntoEnumIterator;

pub struct SettingsClientTab {
    // Fields that taking effect after logout
    compression: bool,

    // Fields that applied after restart
    language: config::Language,
    log_format_choice: String,
    log_level_choice: LevelFilter,

    // Fields that applied by button
    ping_delay_seconds: i64,
    theme: themes::Preference,
    unparsed_frames_drop: bool,
    unparsed_frames_threshold_enabled: bool,
    unparsed_frames_threshold: usize,
}

impl SettingsClientTab {
    pub fn new(ctx: &Context) -> Self {
        Self {
            compression: ctx.client_settings.compression,

            language: ctx.config.language.clone(),
            log_format_choice: ctx.config.log_format.clone(),
            log_level_choice: ctx.config.log_level,

            ping_delay_seconds: ctx.client_settings.sync_delay_seconds,
            theme: ctx.client_settings.theme,

            unparsed_frames_drop: ctx.client_settings.unparsed_frames_drop,
            unparsed_frames_threshold_enabled: ctx
                .client_settings
                .unparsed_frames_threshold
                .is_some(),
            unparsed_frames_threshold: ctx
                .client_settings
                .unparsed_frames_threshold
                .unwrap_or(0),
        }
    }
}

impl SettingsClientTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        const GRID_COLUMNS: usize = 5;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(styles::space::SMALL);
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        spacing::with_temp_y(ui, spacing::GRID, |ui| {
                            Grid::new("Settings.Grid")
                                .striped(false)
                                .num_columns(GRID_COLUMNS)
                                .show(ui, |ui| {
                                    self.save_client_config_view(ui, ctx);
                                    ui.end_row();

                                    self.compression_view(ui, ctx);
                                    ui.end_row();

                                    self.language_view(ui, ctx);
                                    ui.end_row();

                                    self.logs_format_view(ui, ctx);
                                    ui.end_row();

                                    self.logs_level_view(ui, ctx);
                                    ui.end_row();

                                    self.ping_delay_view(ui, ctx);
                                    ui.end_row();

                                    self.theme_view(ui, ctx);
                                    ui.end_row();

                                    self.unparsed_drop_view(ui, ctx);
                                    ui.end_row();

                                    self.unparsed_threshold_view(ui, ctx);
                                    ui.end_row();
                                });
                        });
                    },
                );
            });
    }

    fn save_client_config_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(styles::heading::normal(&t!(
            "Tab.SettingsClient.Label.SaveConfig"
        ))))
        .on_hover_text(t!("Tab.SettingsClient.Hover.SettingSavesConfig"));

        // Invisible element
        ui.label("");

        if ui.button(t!("Button.Save")).clicked() {
            // Fields that taking effect after logout
            ctx.config.compression = ctx.client_settings.compression;

            // Fields that applied by button
            ctx.config.theme = ctx.client_settings.theme;
            ctx.config.sync_delay_seconds = ctx.client_settings.sync_delay_seconds;
            ctx.config.unparsed_frames_drop = ctx.client_settings.unparsed_frames_drop;
            ctx.config.unparsed_frames_threshold =
                ctx.client_settings.unparsed_frames_threshold;

            match ctx.config.save_to_file() {
                Ok(_) => {
                    log::info!("Client Settings: Successfully saved client config.");
                    MessageModal::info(&t!("Message.Success.ClientConfigSaved"))
                        .try_send_by(&ctx.modals_tx);
                },
                Err(err) => {
                    log::error!("Client Settings: Failed to save client config: {}", err);
                    MessageModal::error(&format!(
                        "{} {}",
                        t!("Error.FailedSaveClientConfigIntoFile"),
                        err
                    ))
                    .try_send_by(&ctx.modals_tx);
                },
            };
        }
    }

    fn compression_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.Compression"));
        let not_applied = self.compression != ctx.client_settings.compression;
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedAfterLogout"));

        ui.add(Checkbox::without_text(&mut self.compression));

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!(
                "Client Settings: Compression changed to {}",
                self.compression
            );
            ctx.client_settings.compression = self.compression;
        }
        if ui.button("🔙").clicked() {
            self.compression = ctx.client_settings.compression;
        }
    }

    fn language_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.Language"));
        let not_applied = self.language != ctx.config.language;
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedAfterRestart"));

        ui.with_layout(
            egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ComboBox::from_label("")
                    .selected_text(self.language.localize()) // Display the currently selected option.
                    .show_ui(ui, |ui| {
                        for language in config::Language::iter() {
                            ui.selectable_value(&mut self.language, language.clone(), language.localize());
                        }
                    });
            }
        );

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!("Client Settings: Language changed to {}", self.language);
            ctx.config.language = self.language.clone();
        }

        if ui.button("🔙").clicked() {
            self.language = ctx.config.language.clone();
        }
    }

    fn logs_format_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.LogFormat"));
        let not_applied = !self
            .log_format_choice
            .eq_ignore_ascii_case(&ctx.config.log_format);
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedAfterRestart"));

        ui.add(TextEdit::multiline(&mut self.log_format_choice));

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!(
                "Client Settings: Log Format changed to {}",
                self.log_format_choice
            );
            ctx.config.log_format = self.log_format_choice.clone();
        }

        if ui.button("🔙").clicked() {
            self.log_format_choice = ctx.config.log_format.clone();
        }
    }

    fn logs_level_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.LogLevel"));
        let not_applied = self.log_level_choice != ctx.config.log_level;
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedAfterRestart"));

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            egui::ComboBox::from_id_salt("Settings.Client.Log.Level.ComboBox")
                .selected_text(logging::localize_log_level(&self.log_level_choice))
                .show_ui(ui, |ui| {
                    for level_filter in LevelFilter::iter() {
                        ui.selectable_value(
                            &mut self.log_level_choice,
                            level_filter,
                            logging::localize_log_level(&level_filter),
                        );
                    }
                });
        });

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!("Client Settings: Log Level changed to {}", self.language);
            ctx.config.log_level = self.log_level_choice;
        }

        if ui.button("🔙").clicked() {
            self.log_level_choice = ctx.config.log_level;
        }
    }

    fn ping_delay_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.SyncDelay"));
        let not_applied =
            self.ping_delay_seconds != ctx.client_settings.sync_delay_seconds;
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"));

        ui.add(
            DragValue::new(&mut self.ping_delay_seconds)
                .speed(1)
                .range(1..=i64::MAX)
                .suffix(format!(" {}", t!("Tab.SettingsClient.Suffix.SyncDelay"))),
        );

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!(
                "Client Settings: Sync Delay seconds changed to {}",
                self.ping_delay_seconds
            );
            ctx.client_settings.sync_delay_seconds = self.ping_delay_seconds;
        }

        if ui.button("🔙").clicked() {
            self.ping_delay_seconds = ctx.client_settings.sync_delay_seconds;
        }
    }

    fn theme_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label = styles::heading::normal(&t!("Tab.SettingsClient.Label.Theme"));
        let not_applied = self.theme != ctx.client_settings.theme;
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"));

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            egui::ComboBox::from_id_salt("Settings.Theme.ComboBox")
                .width(200.0)
                .selected_text(self.theme.title())
                .show_ui(ui, |ui| {
                    for theme in themes::Preference::iter() {
                        ui.selectable_value(&mut self.theme, theme, theme.title());
                    }
                });
        });

        if ui.button(t!("Button.Apply")).clicked() {
            ctx.client_settings.theme = self.theme;
            log::info!("Client Settings: Theme changed to {}", self.theme.title());
            ui.ctx()
                .set_style(self.theme.into_aesthetix_theme().custom_style());
        }

        if ui.button("🔙").clicked() {
            self.theme = ctx.client_settings.theme;
        }
    }

    fn unparsed_drop_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label =
            styles::heading::normal(&t!("Tab.SettingsClient.Label.UnparsedFramesDrop"));
        let not_applied =
            self.unparsed_frames_drop != ctx.client_settings.unparsed_frames_drop;
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Label.UnparsedFramesDrop.Note"))
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"));

        ui.add(Checkbox::without_text(&mut self.unparsed_frames_drop));

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!(
                "Client Settings: `Drop Unparsed Frames` changed to {}",
                self.unparsed_frames_drop
            );
            ctx.client_settings.unparsed_frames_drop = self.unparsed_frames_drop;
        }
        if ui.button("🔙").clicked() {
            self.unparsed_frames_drop = ctx.client_settings.unparsed_frames_drop;
        }
    }

    fn unparsed_threshold_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let setting = into_setting(
            self.unparsed_frames_threshold_enabled,
            self.unparsed_frames_threshold,
        );
        let label = styles::heading::normal(&t!(
            "Tab.SettingsClient.Label.UnparsedFramesThreshold"
        ));
        let not_applied = setting != ctx.client_settings.unparsed_frames_threshold;
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Label.UnparsedFramesThreshold.Note"))
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"));

        ui.horizontal_centered(|ui| {
            ui.add(Checkbox::without_text(
                &mut self.unparsed_frames_threshold_enabled,
            ));
            ui.add_enabled(
                self.unparsed_frames_threshold_enabled,
                DragValue::new(&mut self.unparsed_frames_threshold)
                    .speed(1)
                    .range(1..=i64::MAX)
                    .suffix(format!(" {}", t!("Tab.SettingsClient.Suffix.Frames"))),
            );
        });

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!(
                "Client Settings: `Unparsed Frames Threshold` changed to {}:{}",
                self.unparsed_frames_threshold_enabled,
                self.unparsed_frames_threshold,
            );
            ctx.client_settings.unparsed_frames_threshold = setting;
            ctx.net_storage.raw.set_threshold(setting);
        }
        if ui.button("🔙").clicked() {
            self.unparsed_frames_threshold_enabled =
                ctx.client_settings.unparsed_frames_threshold.is_some();
            self.unparsed_frames_threshold =
                ctx.client_settings.unparsed_frames_threshold.unwrap_or(0);
        }

        fn into_setting(is_enabled: bool, amount: usize) -> Option<usize> {
            if is_enabled { Some(amount) } else { None }
        }
    }

    fn label_not_applied(
        ui: &mut egui::Ui, mut label: RichText, is_different: bool,
    ) -> egui::Response {
        if is_different {
            label = label.color(colors::FIELD_NOT_APPLIED);
            ui.add(egui::Label::new(label))
                .on_hover_text(t!("Tab.SettingsClient.Hover.FieldNotApplied"))
        } else {
            ui.add(egui::Label::new(label))
        }
    }
}
