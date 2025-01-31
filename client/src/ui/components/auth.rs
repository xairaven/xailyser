use crate::context::CONTEXT;
use crate::net;
use crate::ui::windows::message::MessageWindow;
use egui::{Button, Grid, TextEdit};
use std::net::{IpAddr, SocketAddr};
use std::thread::JoinHandle;
use thiserror::Error;

#[derive(Default)]
pub struct AuthComponent {
    authenticated: bool,
    pub net_thread: Option<JoinHandle<()>>,

    ip_text_field: String,
    port_text_field: String,
    password_text_field: String,
}

impl AuthComponent {
    pub fn authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.heading("Authentication");
        });

        let available_size = ui.available_size();

        ui.add_space(available_size.y / 3.0);

        ui.columns(3, |columns| {
            columns[1].vertical_centered(|ui| {
                Grid::new("AuthentificationFields")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("IP");
                        ui.add(TextEdit::singleline(&mut self.ip_text_field));
                        ui.end_row();

                        ui.label("Port");
                        ui.add(TextEdit::singleline(&mut self.port_text_field));
                        ui.end_row();

                        ui.label("Password");
                        ui.add(
                            TextEdit::singleline(&mut self.password_text_field)
                                .password(true),
                        );
                        ui.end_row();
                    });

                ui.add_space(available_size.y / 5.0);

                if ui
                    .add_sized([available_size.x / 5.0, 20.0], Button::new("Connect"))
                    .clicked()
                {
                    match self.get_address() {
                        Ok(address) => {
                            self.try_connect(address);
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
    }

    fn try_connect(&mut self, address: SocketAddr) {
        match net::connect(address) {
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
