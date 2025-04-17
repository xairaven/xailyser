use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use crate::ui::themes::ThemePreference;
use crate::utils;
use egui::{Checkbox, DragValue, Grid, RichText, TextEdit};
use log::LevelFilter;
use strum::IntoEnumIterator;

const FIELD_NOT_APPLIED_COLOR: egui::Color32 = egui::Color32::RED;
const FIELD_NOT_APPLIED_HOVER: &str = "This field is not applied at the moment. Also, don’t forget to save the config file if needed.";
const FIELD_RESTART_NEEDED: &str =
    "A restart is needed to apply this field. To save — save the config.";
const FIELD_LOGOUT_NEEDED: &str = "A logout is needed to take effect of this field.";

pub struct SettingsClientTab {
    // Fields that taking effect after logout
    compression: bool,

    // Fields that applied after restart
    log_format_choice: String,
    log_level_choice: LevelFilter,

    // Fields that applied by button
    ping_delay_seconds: i64,
    theme: ThemePreference,
}

impl SettingsClientTab {
    pub fn new(ctx: &Context) -> Self {
        Self {
            compression: ctx.client_settings.compression,

            log_format_choice: ctx.config.log_format.clone(),
            log_level_choice: ctx.config.log_level,

            ping_delay_seconds: ctx.client_settings.sync_delay_seconds,
            theme: ctx.client_settings.theme,
        }
    }
}

impl SettingsClientTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        const GRID_COLUMNS: usize = 4;
        let available_width = ui.available_width();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(20.0);
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    utils::ui::with_temp_spacing_y(ui, 20.0, |ui| {
                        Grid::new("Settings.Grid")
                            .striped(false)
                            .min_col_width(available_width / GRID_COLUMNS as f32)
                            .num_columns(GRID_COLUMNS)
                            .show(ui, |ui| {
                                self.save_client_config_view(ui, ctx);
                                ui.end_row();

                                self.compression_view(ui, ctx);
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
        ui.add(egui::Label::new(
            RichText::new("Save Config:").size(16.0).strong(),
        ));

        // Second Column
        ui.horizontal_centered(|ui| {
            if ui.button("Apply").clicked() {
                // Fields that taking effect after logout
                ctx.config.compression = ctx.client_settings.compression;

                // Fields that applied after restart
                ctx.config.log_format = self.log_format_choice.clone();
                ctx.config.log_level = self.log_level_choice;

                // Fields that applied by button
                ctx.config.theme = ctx.client_settings.theme;
                ctx.config.sync_delay_seconds = ctx.client_settings.sync_delay_seconds;

                let modal = match ctx.config.save_to_file() {
                    Ok(_) => MessageModal::info("Successfully saved client config!"),
                    Err(err) => MessageModal::error(&format!(
                        "Failed to save client config into file! {}",
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
        });
    }

    fn compression_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let mut label = RichText::new("* Compression:").size(16.0).strong();
        if self.compression != ctx.client_settings.compression {
            label = label.color(FIELD_NOT_APPLIED_COLOR);
            ui.add(egui::Label::new(label))
                .on_hover_text(FIELD_NOT_APPLIED_HOVER);
        } else {
            ui.add(egui::Label::new(label))
                .on_hover_text(FIELD_LOGOUT_NEEDED);
        }

        // Second Column
        ui.horizontal_centered(|ui| {
            ui.add(Checkbox::without_text(&mut self.compression));

            if ui.button("Apply").clicked() {
                log::info!(
                    "Client Settings: Compression changed to {}",
                    self.compression
                );
                ctx.client_settings.compression = self.compression;
            }
            if ui.button("🔙").clicked() {
                self.compression = ctx.client_settings.compression;
            }
        });
    }

    fn logs_format_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let different_from_config = !self
            .log_format_choice
            .eq_ignore_ascii_case(&ctx.config.log_format);

        let mut label = RichText::new("* Log Format:").size(16.0).strong();
        if different_from_config {
            label = label.color(FIELD_NOT_APPLIED_COLOR);
        }
        ui.add(egui::Label::new(label))
            .on_hover_text(FIELD_RESTART_NEEDED);

        // Second Column
        ui.horizontal_centered(|ui| {
            ui.add(TextEdit::multiline(&mut self.log_format_choice));

            if ui.button("🔙").clicked() {
                self.log_format_choice = ctx.config.log_format.clone();
            }
        });
    }

    fn logs_level_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let different_from_config = self.log_level_choice != ctx.config.log_level;

        let mut label = RichText::new("* Log Level:").size(16.0).strong();
        if different_from_config {
            label = label.color(FIELD_NOT_APPLIED_COLOR);
        }
        ui.add(egui::Label::new(label))
            .on_hover_text(FIELD_RESTART_NEEDED);

        // Second Column
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("Settings.Client.Log.Level.ComboBox")
                .selected_text(format!("{:?}", &mut self.log_level_choice))
                .show_ui(ui, |ui| {
                    for level_filter in LevelFilter::iter() {
                        ui.selectable_value(
                            &mut self.log_level_choice,
                            level_filter,
                            level_filter.to_string(),
                        );
                    }
                });

            if ui.button("🔙").clicked() {
                self.log_level_choice = ctx.config.log_level;
            }
        });
    }

    fn ping_delay_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let mut label = RichText::new("Sync Delay:").size(16.0).strong();
        if self.ping_delay_seconds != ctx.client_settings.sync_delay_seconds {
            label = label.color(FIELD_NOT_APPLIED_COLOR);
            ui.add(egui::Label::new(label))
                .on_hover_text(FIELD_NOT_APPLIED_HOVER);
        } else {
            ui.add(egui::Label::new(label));
        }

        // Second Column
        ui.horizontal_centered(|ui| {
            ui.add(
                DragValue::new(&mut self.ping_delay_seconds)
                    .speed(1)
                    .range(1..=i64::MAX)
                    .suffix(" seconds"),
            );

            if ui.button("Apply").clicked() {
                log::info!(
                    "Client Settings: Sync Delay seconds changed to {}",
                    self.ping_delay_seconds
                );
                ctx.client_settings.sync_delay_seconds = self.ping_delay_seconds;
            }

            if ui.button("🔙").clicked() {
                self.ping_delay_seconds = ctx.client_settings.sync_delay_seconds;
            }
        });
    }

    fn theme_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let mut label = RichText::new("Theme:").size(16.0).strong();
        if self.theme != ctx.client_settings.theme {
            label = label.color(FIELD_NOT_APPLIED_COLOR);
            ui.add(egui::Label::new(label))
                .on_hover_text(FIELD_NOT_APPLIED_HOVER);
        } else {
            ui.add(egui::Label::new(label));
        }

        // Second Column
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("Settings.Theme.ComboBox")
                .width(200.0)
                .selected_text(self.theme.title())
                .show_ui(ui, |ui| {
                    for theme in ThemePreference::iter() {
                        ui.selectable_value(&mut self.theme, theme, theme.title());
                    }
                });

            if ui.button("Apply").clicked() {
                ctx.client_settings.theme = self.theme;
                log::info!("Client Settings: Theme changed to {}", self.theme.title());
                ui.ctx()
                    .set_style(self.theme.into_aesthetix_theme().custom_style());
            }

            if ui.button("🔙").clicked() {
                self.theme = ctx.client_settings.theme;
            }
        });
    }
}
