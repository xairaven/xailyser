use crate::context::CONTEXT;
use crate::net;
use crate::ui::windows::message::MessageWindow;
use egui::{Grid, RichText, TextEdit};
use std::net::{IpAddr, SocketAddr};
use std::thread::JoinHandle;
use thiserror::Error;

#[derive(Default)]
pub struct AuthRoot {
    authenticated: bool,
    pub net_thread: Option<JoinHandle<()>>,

    ip_text_field: String,
    port_text_field: String,
    password_text_field: String,
}

impl AuthRoot {
    pub fn authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
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

                ui.add_space(window_height / 4.0);

                ui.vertical_centered_justified(|ui| {
                    if ui.button("CONNECT").clicked() {
                        match self.get_address() {
                            Ok(address) => {
                                self.try_connect(
                                    address,
                                    &self.password_text_field.clone(),
                                );
                            },
                            Err(err) => {
                                if let Ok(guard) = CONTEXT.try_lock() {
                                    let window = MessageWindow::error(&err.to_string());
                                    let _ = guard.windows_tx.send(Box::new(window));
                                }
                            },
                        }
                    }
                });
            });
        });
    }

    fn try_connect(&mut self, address: SocketAddr, password: &str) {
        match net::connect(address, password) {
            Ok(handle) => {
                self.net_thread = Some(handle);
                self.authenticated = true;
            },
            Err(err) => {
                let message = match err.additional_info() {
                    None => format!("{}.", err),
                    Some(info) => format!("{}.\n{}", err, info),
                };

                if let Ok(guard) = CONTEXT.try_lock() {
                    let window = MessageWindow::error(&message);
                    let _ = guard.windows_tx.send(Box::new(window));
                }
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
}

#[derive(Error, Debug)]
enum AddressConversionError {
    #[error("Failed to parse IP address")]
    WrongIpAddress,

    #[error("Failed to parse port")]
    WrongPort,
}
