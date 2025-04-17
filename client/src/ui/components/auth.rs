use crate::context::Context;
use crate::ui::components::preauth_client_settings::PreAuthClientSettingsComponent;
use crate::ui::modals::message::MessageModal;
use crate::ws;
use crate::ws::WsHandler;
use egui::{Grid, RichText, TextEdit};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use thiserror::Error;

pub struct AuthComponent {
    pub net_thread: Option<JoinHandle<()>>,
    pub pre_auth_settings_component: PreAuthClientSettingsComponent,

    authenticated: bool,

    ip_text_field: String,
    port_text_field: String,
    password_text_field: String,
}

impl AuthComponent {
    pub fn new(ctx: &Context) -> Self {
        Self {
            pre_auth_settings_component: PreAuthClientSettingsComponent::new(ctx),

            net_thread: None,
            authenticated: false,
            ip_text_field: "".to_string(),
            port_text_field: "".to_string(),
            password_text_field: "".to_string(),
        }
    }

    pub fn authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        // Show settings if opened instead of auth window
        if self.pre_auth_settings_component.is_opened() {
            self.pre_auth_settings_component.show(ui, ctx);
            return;
        }

        let window_height = ui.available_size().y;

        ui.columns(3, |columns| {
            const MAIN_COLUMN: usize = 1;
            const RIGHT_COLUMN: usize = 2;

            columns[MAIN_COLUMN].vertical_centered(|ui| {
                ui.add_space(window_height / 6.0);

                ui.vertical_centered_justified(|ui| {
                    ui.label(RichText::new("Login").size(26.0));
                });

                ui.add_space(window_height / 6.0);

                Grid::new("AuthenticationFields")
                    .num_columns(2)
                    .striped(false)
                    .spacing([20.0, 20.0])
                    .show(ui, |ui| {
                        ui.label("IP:");
                        ui.add(
                            TextEdit::singleline(&mut self.ip_text_field)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.label("Port:");
                        ui.add(
                            TextEdit::singleline(&mut self.port_text_field)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.label("Password:");
                        ui.add(
                            TextEdit::singleline(&mut self.password_text_field)
                                .password(true)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();
                    });

                ui.add_space(window_height / 6.0);

                ui.vertical_centered_justified(|ui| {
                    if ui.button("CONNECT").clicked() {
                        match self.get_address() {
                            Ok(address) => {
                                self.try_connect(
                                    ctx,
                                    address,
                                    &self.password_text_field.clone(),
                                );
                            },
                            Err(err) => {
                                let modal = MessageModal::error(&err.to_string());
                                let _ = ctx.modals_tx.send(Box::new(modal));
                            },
                        }
                    }
                });
            });

            columns[RIGHT_COLUMN].with_layout(
                egui::Layout::right_to_left(egui::Align::Min),
                |ui| {
                    if ui.button("⚙").clicked() {
                        self.pre_auth_settings_component.open();
                    }
                },
            );
        });
    }

    fn try_connect(&mut self, ctx: &Context, address: SocketAddr, password: &str) {
        match ws::connect(address, password, ctx.client_settings.compression) {
            Ok(stream) => {
                let mut ws_handler = WsHandler {
                    compression: ctx.client_settings.compression,
                    shutdown_flag: Arc::clone(&ctx.shutdown_flag),
                    stream,
                    server_response_tx: ctx.server_response_tx.clone(),
                    ui_client_requests_rx: ctx.ui_client_requests_rx.clone(),
                };

                let handle = thread::Builder::new()
                    .name("WS-Thread".to_string())
                    .spawn(move || {
                        ws_handler.send_receive_messages();
                    })
                    .unwrap_or_else(|err| {
                        log::error!("Failed to spawn WS thread: {}", err);
                        std::process::exit(1);
                    });

                self.net_thread = Some(handle);
                self.authenticated = true;
            },
            Err(err) => {
                let message = match err.additional_info() {
                    None => format!("{}.", err),
                    Some(info) => format!("{}.\n{}", err, info),
                };

                let modal = MessageModal::error(&message);
                let _ = ctx.modals_tx.send(Box::new(modal));
            },
        }
    }

    fn get_address(&self) -> Result<SocketAddr, AddressConversionError> {
        let ip_address: IpAddr = self
            .ip_text_field
            .trim()
            .parse()
            .map_err(|_| AddressConversionError::WrongIpAddress)?;
        let port: u16 = self
            .port_text_field
            .trim()
            .parse()
            .map_err(|_| AddressConversionError::WrongPort)?;

        Ok(SocketAddr::new(ip_address, port))
    }

    pub fn logout(&mut self, ctx: &Context) {
        self.authenticated = false;
        self.net_thread = None;
        self.pre_auth_settings_component.update_tab(ctx);
    }
}

#[derive(Error, Debug)]
enum AddressConversionError {
    #[error("Failed to parse IP address")]
    WrongIpAddress,

    #[error("Failed to parse port")]
    WrongPort,
}
