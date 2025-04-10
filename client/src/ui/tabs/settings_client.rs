use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use crate::ui::themes::ThemePreference;
use egui::{DragValue, Grid, RichText, TextEdit};
use log::LevelFilter;
use strum::IntoEnumIterator;

const FIELD_NOT_APPLIED_COLOR: egui::Color32 = egui::Color32::RED;
const FIELD_NOT_APPLIED_HOVER: &str = "This field is not applied at the moment.";

pub struct SettingsClientTab {
    log_level_choice: LevelFilter,
    log_format_choice: String,
    ping_delay_seconds: i64,
}

impl SettingsClientTab {
    pub fn new(ctx: &Context) -> Self {
        Self {
            log_level_choice: ctx.config.log_level,
            log_format_choice: ctx.config.log_format.clone(),
            ping_delay_seconds: ctx.config.sync_delay_seconds,
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
                    Grid::new("Settings.Grid")
                        .striped(false)
                        .min_col_width(available_width / GRID_COLUMNS as f32)
                        .num_columns(GRID_COLUMNS)
                        .show(ui, |ui| {
                            self.save_client_config_view(ui, ctx);
                            ui.end_row();

                            self.theme_view(ui, ctx);
                            ui.end_row();

                            self.logs_level_view(ui, ctx);
                            ui.end_row();

                            self.logs_format_view(ui, ctx);
                            ui.end_row();

                            self.ping_delay_view(ui, ctx);
                            ui.end_row();
                        });
                },
            );
        });
    }

    fn theme_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(
            RichText::new("Theme:").size(16.0).strong(),
        ));

        egui::ComboBox::from_id_salt("Settings.Theme.ComboBox")
            .width(200.0)
            .selected_text(ctx.config.theme.title())
            .show_ui(ui, |ui| {
                for theme in ThemePreference::iter() {
                    let res: egui::Response =
                        ui.selectable_value(&mut ctx.config.theme, theme, theme.title());
                    if res.changed() {
                        log::info!("Theme changed to {}", theme.title());
                        ui.ctx()
                            .set_style(theme.into_aesthetix_theme().custom_style());
                    }
                }
            });
    }

    fn save_client_config_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(
            RichText::new("Save Config:").size(16.0).strong(),
        ));

        if ui.button("Apply").clicked() {
            let modal = match ctx.config.save_to_file() {
                Ok(_) => MessageModal::info("Successfully saved client config!"),
                Err(err) => {
                    MessageModal::error(&format!("Failed to save client config! {}", err))
                },
            };
            match ctx.modals_tx.try_send(Box::new(modal)) {
                Ok(_) => log::info!("Requested saving client config. Saved"),
                Err(_) => log::error!("Requested saving client config. Failed to save."),
            }
        }
    }

    fn logs_level_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let mut label = RichText::new("Log Level:").size(16.0).strong();
        if self.log_level_choice != ctx.config.log_level {
            label = label.color(FIELD_NOT_APPLIED_COLOR);
            ui.add(egui::Label::new(label))
                .on_hover_text(FIELD_NOT_APPLIED_HOVER);
        } else {
            ui.add(egui::Label::new(label));
        }

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

        if ui.button("Apply").clicked() {
            ctx.config.log_level = self.log_level_choice;
            let modal = MessageModal::info(
                "Successfully changed log level! Don't forget to save configuration!",
            );
            match ctx.modals_tx.try_send(Box::new(modal)) {
                Ok(_) => log::info!("Requested changing log level. Success"),
                Err(_) => log::error!("Requested changing log level. Failure"),
            }
        }
    }

    fn logs_format_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let mut label = RichText::new("Log Format:").size(16.0).strong();
        if !self
            .log_format_choice
            .eq_ignore_ascii_case(&ctx.config.log_format)
        {
            label = label.color(FIELD_NOT_APPLIED_COLOR);
            ui.add(egui::Label::new(label))
                .on_hover_text(FIELD_NOT_APPLIED_HOVER);
        } else {
            ui.add(egui::Label::new(label));
        }

        ui.add(TextEdit::multiline(&mut self.log_format_choice));

        if ui.button("Apply").clicked() {
            ctx.config.log_format = self.log_format_choice.clone();
            let modal = MessageModal::info(
                "Successfully changed log format! Don't forget to save configuration!",
            );
            match ctx.modals_tx.try_send(Box::new(modal)) {
                Ok(_) => log::info!("Requested changing log format. Success"),
                Err(_) => log::error!("Requested changing log format. Failure"),
            }
        }
    }

    fn ping_delay_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let mut label = RichText::new("Sync Delay:").size(16.0).strong();
        if self.ping_delay_seconds != ctx.config.sync_delay_seconds {
            label = label.color(FIELD_NOT_APPLIED_COLOR);
            ui.add(egui::Label::new(label))
                .on_hover_text(FIELD_NOT_APPLIED_HOVER);
        } else {
            ui.add(egui::Label::new(label));
        }

        ui.add(
            DragValue::new(&mut self.ping_delay_seconds)
                .speed(1)
                .range(1..=i64::MAX)
                .suffix(" seconds"),
        );

        if ui.button("Apply").clicked() {
            ctx.config.sync_delay_seconds = self.ping_delay_seconds;
            let modal = MessageModal::info(
                "Successfully changed ping delay! Don't forget to save configuration!",
            );
            match ctx.modals_tx.try_send(Box::new(modal)) {
                Ok(_) => log::info!("Requested changing sync delay. Success"),
                Err(_) => log::error!("Requested changing sync delay. Failure"),
            }
        }
    }
}
