use crate::communication::request::UiClientRequest;
use crate::context::Context;
use crate::ui::styles;
use crate::ui::styles::{colors, spacing};
use crate::ui::tabs::Tab;
use chrono::{DateTime, Local};
use common::messages::Request;
use egui::{Grid, RichText, TextBuffer, TextEdit};

#[derive(Default)]
pub struct SettingsServerTab {
    pub reboot_requested: bool, // To logout after reboot
    reboot_confirm: bool,       // To show confirmation

    password_field: String,
    interface_current: Option<String>,

    last_request: Option<DateTime<Local>>, // For "Last Updated:"
}

impl SettingsServerTab {
    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        self.tab_heading(ui);

        const GRID_COLUMNS: usize = 4;
        let available_width = ui.available_width();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(styles::space::SMALL);
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    Grid::new("Settings.Grid")
                        .striped(false)
                        .min_col_width(available_width / GRID_COLUMNS as f32)
                        .num_columns(GRID_COLUMNS)
                        .show(ui, |ui| {
                            self.request_settings_view(ui, ctx);
                            ui.end_row();

                            self.save_server_config_view(ui, ctx);
                            ui.end_row();

                            self.reboot_view(ui, ctx);
                            ui.end_row();

                            self.compression_view(ui, ctx);
                            ui.end_row();

                            self.change_password_view(ui, ctx);
                            ui.end_row();

                            self.sending_unparsed_frames_view(ui, ctx);
                            ui.end_row();
                        });

                    self.interfaces_view(ui, ctx);
                },
            );
        });
    }

    fn tab_heading(&self, ui: &mut egui::Ui) {
        ui.add_space(styles::space::TAB);
        ui.heading(
            RichText::new(Tab::ServerSettings.to_string().as_str())
                .size(styles::heading::HUGE),
        );
    }

    fn request_settings_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        if ui
            .button(t!("Tab.SettingsServer.Label.RequestSettings"))
            .clicked()
        {
            self.request_server_settings(ctx);
        }

        let req_upd_timestamp =
            match (&self.last_request, &ctx.settings_server.last_updated) {
                (Some(req), Some(upd)) => {
                    let formatted = req.format(styles::TIME_FORMAT).to_string();
                    let color = if req > upd {
                        colors::OUTDATED
                    } else {
                        colors::UPDATED
                    };
                    RichText::new(formatted).color(color)
                },
                (None, Some(upd)) => {
                    let formatted = upd.format(styles::TIME_FORMAT).to_string();
                    RichText::new(formatted).color(colors::UPDATED)
                },
                (Some(req), None) => {
                    let formatted = req.format(styles::TIME_FORMAT).to_string();
                    RichText::new(formatted).color(colors::OUTDATED)
                },
                (None, None) => {
                    RichText::new(t!("Text.LastUpdate.Never")).color(colors::OUTDATED)
                },
            };
        ui.label(req_upd_timestamp);
    }

    fn save_server_config_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(styles::heading::normal(&t!(
            "Tab.SettingsServer.Label.SaveConfig"
        ))));

        if ui.button(t!("Button.Apply")).clicked() {
            if let Err(err) = ctx
                .ui_client_requests_tx
                .try_send(UiClientRequest::Request(Request::SaveConfig))
            {
                log::error!("Failed to send command (Save Config): {}", err);
            } else {
                log::info!("UI -> WS: Sent 'Save Config' command.");
            }
            self.request_server_settings(ctx);
        }
    }

    fn reboot_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(styles::heading::normal(&t!(
            "Tab.SettingsServer.Label.RestartServer"
        ))))
        .on_hover_text(t!("Tab.SettingsServer.Note.RestartServer"));

        if !self.reboot_confirm {
            if ui.button(t!("Button.Apply")).clicked() {
                self.reboot_confirm = true;
            }
        } else {
            if ui.button(t!("Button.Confirm")).clicked() {
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

            if ui.button(t!("Button.Cancel")).clicked() {
                self.reboot_confirm = false;
            }
        }
    }

    fn compression_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let differ = ctx.settings_server.compression_active
            != ctx.settings_server.compression_config;

        ui.add(egui::Label::new(styles::heading::normal(&t!(
            "Tab.SettingsServer.Label.Compression"
        ))));
        let is_enabled_text =
            styles::text::is_enabled(ctx.settings_server.compression_active);
        Self::different_from_config(ui, is_enabled_text, differ);

        // We don't care what active field is - changes take effect only on config
        if ui
            .button(styles::text::action(ctx.settings_server.compression_config))
            .clicked()
        {
            let _ = ctx.ui_client_requests_tx.try_send(UiClientRequest::Request(
                Request::SetCompression(!ctx.settings_server.compression_config),
            ));
            self.request_server_settings(ctx);
        }
    }

    fn change_password_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.add(egui::Label::new(styles::heading::normal(&t!(
            "Tab.SettingsServer.Label.ChangePassword"
        ))));

        ui.add(TextEdit::singleline(&mut self.password_field));

        if ui.button(t!("Button.Apply")).clicked() {
            let _ = ctx.ui_client_requests_tx.try_send(UiClientRequest::Request(
                Request::ChangePassword(self.password_field.take()),
            ));
        }
    }

    fn interfaces_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        ui.collapsing(
            styles::heading::normal(&t!("Tab.SettingsServer.Label.Interfaces")),
            |ui| {
                spacing::with_temp_y(ui, 4.0, |ui| {
                    Grid::new("Settings.Interfaces.Status.Grid")
                        .striped(false)
                        .num_columns(3)
                        .show(ui, |ui| {
                            ui.label(format!("{}:", t!("Text.Active")));
                            ui.label(
                                RichText::new(
                                    ctx.settings_server
                                        .interface_active
                                        .as_ref()
                                        .unwrap_or(&t!("Text.None").to_string()),
                                )
                                .strong(),
                            );
                            ui.end_row();

                            // Optional "Config Interface" label
                            if ctx.settings_server.interface_active.as_ref()
                                != ctx.settings_server.interface_config.as_ref()
                            {
                                ui.label(format!(
                                    "{}:",
                                    t!("Tab.SettingsServer.Label.InterfaceConfig")
                                ));
                                if let Some(config_interface) =
                                    &ctx.settings_server.interface_config
                                {
                                    ui.label(RichText::new(config_interface).italics());
                                } else {
                                    ui.label(t!("Text.None"));
                                }
                                ui.end_row();
                            }

                            // Chosen interface (clicked on button)
                            if let Some(chosen) = &self.interface_current {
                                ui.label(format!("{}:", t!("Text.Chosen")));
                                ui.label(RichText::new(chosen).italics());

                                if ui.button(t!("Button.Apply")).clicked() {
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
                                    self.request_server_settings(ctx);
                                    self.interface_current = None;
                                }

                                if ui.button(t!("Button.Reset")).clicked() {
                                    self.interface_current = None;
                                }
                                ui.end_row();
                            }

                            ui.end_row();
                        });
                });

                ui.add_space(16.0);

                if !ctx.settings_server.interfaces_available.is_empty() {
                    ui.label(format!(
                        "{}:",
                        t!("Tab.SettingsServer.Label.Interfaces.Available")
                    ));
                    ui.vertical_centered_justified(|ui| {
                        for interface in &ctx.settings_server.interfaces_available {
                            if ui.button(RichText::new(interface).monospace()).clicked() {
                                self.interface_current = Some(interface.to_string());
                            }
                        }
                    });
                } else {
                    ui.label(format!(
                        "{}:\t—",
                        "Tab.SettingsServer.Label.Interfaces.Available"
                    ));
                }
            },
        );
    }

    fn sending_unparsed_frames_view(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let differ = ctx.settings_server.send_unparsed_frames_active
            != ctx.settings_server.send_unparsed_frames_config;

        ui.add(egui::Label::new(styles::heading::normal(&t!(
            "Tab.SettingsServer.Label.SendUnparsedFrames"
        ))));
        let is_enabled_text =
            styles::text::is_enabled(ctx.settings_server.send_unparsed_frames_active);
        Self::different_from_config(ui, is_enabled_text, differ);

        // We don't care what active field is - changes take effect only on config
        if ui
            .button(styles::text::action(
                ctx.settings_server.send_unparsed_frames_config,
            ))
            .clicked()
        {
            let _ = ctx.ui_client_requests_tx.try_send(UiClientRequest::Request(
                Request::SetSendUnparsedFrames(
                    !ctx.settings_server.send_unparsed_frames_config,
                ),
            ));
            self.request_server_settings(ctx);
        }
    }

    fn different_from_config(
        ui: &mut egui::Ui, label: RichText, is_different: bool,
    ) -> egui::Response {
        if is_different {
            ui.add(egui::Label::new(label.italics()))
                .on_hover_text(t!("Tab.SettingsServer.Hover.FieldDifferFromConfig"))
        } else {
            ui.add(egui::Label::new(label))
        }
    }

    fn request_server_settings(&mut self, ctx: &mut Context) {
        self.last_request = Some(Local::now());
        let result = ctx
            .ui_client_requests_tx
            .try_send(UiClientRequest::Request(Request::ServerSettings));
        if let Err(err) = result {
            log::error!("Server Settings: {}", err);
        }
    }
}
