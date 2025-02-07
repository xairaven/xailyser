use crate::context::Context;
use crate::ui::modals::message::MessageModal;
use crate::ws;
use egui::{Grid, RichText, TextEdit};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use thiserror::Error;

#[derive(Default)]
pub struct AuthComponent {
    pub net_thread: Option<JoinHandle<()>>,

    authenticated: bool,

    ip_text_field: String,
    port_text_field: String,
    password_text_field: String,
}

impl AuthComponent {
    pub fn authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &mut Context) {
        let window_height = ui.available_size().y;

        ui.columns(3, |columns| {
            columns[1].vertical_centered(|ui| {
                ui.add_space(window_height / 6.0);

                ui.vertical_centered_justified(|ui| {
                    ui.label(
                        RichText::new("Login")
                            .color(egui::Color32::WHITE)
                            .size(26.0),
                    );
                });

                ui.add_space(window_height / 6.0);

                Grid::new("AuthenticationFields")
                    .num_columns(2)
                    .striped(false)
                    .spacing([20.0, 20.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("IP:").color(egui::Color32::WHITE));
                        ui.add(
                            TextEdit::singleline(&mut self.ip_text_field)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.label(RichText::new("Port:").color(egui::Color32::WHITE));
                        ui.add(
                            TextEdit::singleline(&mut self.port_text_field)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.label(RichText::new("Password:").color(egui::Color32::WHITE));
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
        });
    }

    fn try_connect(&mut self, ctx: &Context, address: SocketAddr, password: &str) {
        match ws::connect(address, password) {
            Ok(stream) => {
                let server_response_tx = ctx.server_response_tx.clone();
                let ui_client_requests_rx = ctx.ui_client_requests_rx.clone();
                let shutdown_flag = Arc::clone(&ctx.shutdown_flag);
                let handle = thread::spawn(move || {
                    ws::send_receive_messages(
                        stream,
                        server_response_tx,
                        ui_client_requests_rx,
                        shutdown_flag,
                    );
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

    pub fn logout(&mut self) {
        self.authenticated = false;
        self.net_thread = None;
    }
}

#[derive(Error, Debug)]
enum AddressConversionError {
    #[error("Failed to parse IP address")]
    WrongIpAddress,

    #[error("Failed to parse port")]
    WrongPort,
}
