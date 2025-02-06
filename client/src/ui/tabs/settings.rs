use crate::communication::request::UiClientRequest;
use crate::context::Context;
use crate::ui::themes::ThemePreference;
use chrono::{DateTime, Local};
use egui::{Color32, Grid, RichText};
use strum::IntoEnumIterator;
use xailyser_common::messages::Request;

#[derive(Default)]
pub struct SettingsTab {
    reboot_requested: bool, // To show confirmation

    interface_current: Option<String>,
    interfaces_last_request: Option<DateTime<Local>>, // For "Last Updated:"
}

impl SettingsTab {
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
                            self.theme_view(ui, ctx);
                            ui.end_row();

                            self.reboot_view(ui, ctx);
                            ui.end_row();
                        });

                    self.interfaces_view(ui, ctx);
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
            .selected_text(ctx.active_theme.title())
            .show_ui(ui, |ui| {
                for theme in ThemePreference::iter() {
                    let res: egui::Response =
                        ui.selectable_value(&mut ctx.active_theme, theme, theme.title());
                    if res.changed() {
                        log::info!("Theme changed to {}", theme.title());
                        ui.ctx()
                            .set_style(theme.into_aesthetix_theme().custom_style());
                    }
                }
            });
    }

    fn reboot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(
            RichText::new("Restart the server:")
                .size(16.0)
                .strong(),
        )).on_hover_text("After confirmation, you may not receive a message about the reboot.\nMonitor the server status.");

        if !self.reboot_requested {
            if ui.button("Apply").clicked() {
                self.reboot_requested = true;
            }
        } else {
            if ui.button("CONFIRM").clicked() {
                self.reboot_requested = false;

                if let Err(err) = ctx
                    .ui_client_requests_tx
                    .try_send(UiClientRequest::Request(Request::Reboot))
                {
                    log::error!("Failed to send command (Reboot): {}", err);
                } else {
                    log::info!("UI -> WS: Sent reboot command.");
                }
            }

            if ui.button("Cancel").clicked() {
                self.reboot_requested = false;
            }
        }

        ui.end_row();
    }

    fn interfaces_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.collapsing(RichText::new("Interfaces:").size(16.0).strong(), |ui| {
            Grid::new("Settings.Interfaces.Status.Grid")
                .striped(false)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.label("Active:");
                    ui.label(
                        RichText::new(
                            ctx.interface_active.clone().unwrap_or("None".to_string()),
                        )
                        .strong(),
                    );
                    ui.end_row();

                    if let Some(chosen) = &self.interface_current {
                        ui.label("Chosen:");
                        ui.label(RichText::new(chosen).italics());

                        if ui.button("Apply").clicked() {
                            if let Err(err) = ctx.ui_client_requests_tx.try_send(
                                UiClientRequest::Request(Request::SetInterface(
                                    chosen.clone(),
                                )),
                            ) {
                                log::error!(
                                    "Failed to send request (SetInterface): {}",
                                    err
                                );
                            }
                            self.interface_current = None;
                        }

                        if ui.button("Reset").clicked() {
                            self.interface_current = None;
                        }
                        ui.end_row();
                    }

                    ui.end_row();
                    ui.label("Last Request Update:\t");
                    if let Some(time) = &self.interfaces_last_request {
                        ui.colored_label(
                            Color32::GREEN,
                            time.format("%m/%d %H:%M:%S").to_string(),
                        );
                    } else {
                        ui.colored_label(Color32::RED, "Never");
                    }
                    ui.end_row();

                    ui.label("Request List:");
                    if ui.button("Request").clicked() {
                        self.interfaces_last_request = Some(Local::now());
                        let _ = ctx.ui_client_requests_tx.try_send(
                            UiClientRequest::Request(Request::RequestInterfaces),
                        );
                    }
                });

            ui.add_space(16.0);

            if !ctx.interfaces_available.is_empty() {
                ui.label("Available Interfaces:");
                ui.vertical_centered_justified(|ui| {
                    for interface in &ctx.interfaces_available {
                        if ui.button(RichText::new(interface).monospace()).clicked() {
                            self.interface_current = Some(interface.to_string());
                        }
                    }
                });
            } else {
                ui.label("Available Interfaces:\t—");
            }
        });
    }
}
