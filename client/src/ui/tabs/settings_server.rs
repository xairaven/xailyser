use crate::communication::request::UiClientRequest;
use crate::context::Context;
use crate::utils;
use chrono::{DateTime, Local};
use common::messages::Request;
use egui::{Color32, Grid, RichText, TextBuffer, TextEdit};

#[derive(Default)]
pub struct SettingsServerTab {
    pub reboot_requested: bool, // To logout after reboot
    reboot_confirm: bool,       // To show confirmation

    password_field: String,

    interface_current: Option<String>,
    interfaces_last_request: Option<DateTime<Local>>, // For "Last Updated:"
}

impl SettingsServerTab {
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
                            self.save_server_config_view(ui, ctx);
                            ui.end_row();

                            self.reboot_view(ui, ctx);
                            ui.end_row();

                            self.change_password_view(ui, ctx);
                            ui.end_row();
                        });

                    self.interfaces_view(ui, ctx);
                },
            );
        });
    }

    fn save_server_config_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(
            RichText::new("Save Server Config:").size(16.0).strong(),
        ));

        if ui.button("Apply").clicked() {
            if let Err(err) = ctx
                .ui_client_requests_tx
                .try_send(UiClientRequest::Request(Request::SaveConfig))
            {
                log::error!("Failed to send command (Save Config): {}", err);
            } else {
                log::info!("UI -> WS: Sent 'Save Config' command.");
            }
        }
    }

    fn reboot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(
            RichText::new("Restart the server:")
                .size(16.0)
                .strong(),
        )).on_hover_text("After confirmation, you may not receive a message about the reboot.\nMonitor the server status.");

        if !self.reboot_confirm {
            if ui.button("Apply").clicked() {
                self.reboot_confirm = true;
            }
        } else {
            if ui.button("CONFIRM").clicked() {
                self.reboot_confirm = false;
                self.reboot_requested = true;

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
                self.reboot_confirm = false;
            }
        }
    }

    fn change_password_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(
            RichText::new("Change Password:").size(16.0).strong(),
        ));

        ui.add(TextEdit::singleline(&mut self.password_field));

        if ui.button("Apply").clicked() {
            let _ = ctx.ui_client_requests_tx.try_send(UiClientRequest::Request(
                Request::ChangePassword(self.password_field.take()),
            ));
        }
    }

    fn interfaces_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.collapsing(RichText::new("Interfaces:").size(16.0).strong(), |ui| {
            utils::ui::with_temp_spacing_y(ui, 4.0, |ui| {
                Grid::new("Settings.Interfaces.Status.Grid")
                    .striped(false)
                    .num_columns(3)
                    .show(ui, |ui| {
                        ui.label("Active:");
                        ui.label(
                            RichText::new(
                                ctx.interface_active
                                    .as_ref()
                                    .unwrap_or(&"None".to_string()),
                            )
                            .strong(),
                        );
                        ui.end_row();

                        if let (Some(active_interface), Some(config_interface)) = (
                            ctx.interface_active.as_ref(),
                            ctx.interface_active_config.as_ref(),
                        ) {
                            if active_interface != config_interface {
                                ui.label("Config Interface:");
                                ui.label(RichText::new(config_interface).italics());
                                ui.end_row();
                            }
                        }

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
                                self.request_interfaces(ctx);
                                self.interface_current = None;
                            }

                            if ui.button("Reset").clicked() {
                                self.interface_current = None;
                            }
                            ui.end_row();
                        }

                        ui.end_row();
                        ui.label("Last Request Update:\t");
                        {
                            let mut text = RichText::new("Never").color(Color32::RED);

                            let last_request = &self.interfaces_last_request;
                            let last_update = &ctx.interfaces_last_updated;

                            if let (Some(last_request), Some(last_update)) =
                                (last_request, last_update)
                            {
                                if last_request > last_update {
                                    text = RichText::new(
                                        last_request.format("%m/%d %H:%M:%S").to_string(),
                                    )
                                    .color(Color32::RED);
                                } else {
                                    text = RichText::new(
                                        last_update.format("%m/%d %H:%M:%S").to_string(),
                                    )
                                    .color(Color32::GREEN);
                                }
                            } else if let Some(last_update) = &last_update {
                                text = RichText::new(
                                    last_update.format("%m/%d %H:%M:%S").to_string(),
                                )
                                .color(Color32::GREEN);
                            } else if let Some(last_request) = &last_request {
                                text = RichText::new(
                                    last_request.format("%m/%d %H:%M:%S").to_string(),
                                )
                                .color(Color32::RED);
                            }

                            ui.label(text);
                        }
                        ui.end_row();

                        ui.label("Request List:");
                        if ui.button("Request").clicked() {
                            self.interfaces_last_request = Some(Local::now());
                            self.request_interfaces(ctx);
                        }
                    });
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

    fn request_interfaces(&self, ctx: &mut Context) {
        let _ = ctx
            .ui_client_requests_tx
            .try_send(UiClientRequest::Request(Request::RequestInterfaces));
        let _ = ctx
            .ui_client_requests_tx
            .try_send(UiClientRequest::Request(Request::RequestActiveInterface));
        let _ = ctx
            .ui_client_requests_tx
            .try_send(UiClientRequest::Request(Request::RequestConfigInterface));
    }
}
