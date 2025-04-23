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
    drop_unparsed_frames: bool,
    ping_delay_seconds: i64,
    theme: themes::Preference,
}

impl SettingsClientTab {
    pub fn new(ctx: &Context) -> Self {
        Self {
            compression: ctx.client_settings.compression,

            language: ctx.config.language.clone(),
            log_format_choice: ctx.config.log_format.clone(),
            log_level_choice: ctx.config.log_level,

            drop_unparsed_frames: ctx.client_settings.drop_unparsed_frames,
            ping_delay_seconds: ctx.client_settings.sync_delay_seconds,
            theme: ctx.client_settings.theme,
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

                                    self.drop_unparsed_view(ui, ctx);
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
            ctx.config.drop_unparsed_frames = ctx.client_settings.drop_unparsed_frames;
            ctx.config.theme = ctx.client_settings.theme;
            ctx.config.sync_delay_seconds = ctx.client_settings.sync_delay_seconds;

            let modal = match ctx.config.save_to_file() {
                Ok(_) => MessageModal::info(&t!("Message.Success.ClientConfigSaved")),
                Err(err) => MessageModal::error(&format!(
                    "{} {}",
                    t!("Error.FailedSaveClientConfigIntoFile"),
                    err
                )),
            };
            match ctx.modals_tx.try_send(Box::new(modal)) {
                Ok(_) => log::info!("Requested saving client config. Saved"),
                Err(_) => {
                    log::error!("Requested saving client config. Failed to save.")
                },
            }
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

    fn drop_unparsed_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let label =
            styles::heading::normal(&t!("Tab.SettingsClient.Label.DropUnparsedFrames"));
        let not_applied =
            self.drop_unparsed_frames != ctx.client_settings.drop_unparsed_frames;
        Self::label_not_applied(ui, label, not_applied)
            .on_hover_text(t!("Tab.SettingsClient.Label.DropUnparsedFrames.Note"))
            .on_hover_text(t!("Tab.SettingsClient.Note.FieldAppliedImmediately"));

        ui.add(Checkbox::without_text(&mut self.drop_unparsed_frames));

        if ui.button(t!("Button.Apply")).clicked() {
            log::info!(
                "Client Settings: `Drop Unparsed Frames` changed to {}",
                self.drop_unparsed_frames
            );
            ctx.client_settings.drop_unparsed_frames = self.drop_unparsed_frames;
        }
        if ui.button("🔙").clicked() {
            self.drop_unparsed_frames = ctx.client_settings.drop_unparsed_frames;
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
