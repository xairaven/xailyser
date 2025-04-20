use crate::context::Context;
use crate::profiles::Profile;
use crate::ui::components::connection_profiles::ConnectionProfilesComponent;
use crate::ui::components::preauth_client_settings::PreAuthClientSettingsComponent;
use crate::ui::modals::message::MessageModal;
use crate::ws;
use crate::ws::WsHandler;
use egui::{Grid, RichText, TextEdit};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub struct AuthComponent {
    // Net thread
    pub net_thread: Option<JoinHandle<()>>,

    // Components
    pub connection_profiles_component: ConnectionProfilesComponent,
    pub pre_auth_settings_component: PreAuthClientSettingsComponent,

    // Internal fields
    authenticated: bool,
    auth_fields: AuthFields,
}

impl AuthComponent {
    pub fn new(ctx: &Context) -> Self {
        Self {
            net_thread: None,

            connection_profiles_component: Default::default(),
            pre_auth_settings_component: PreAuthClientSettingsComponent::new(ctx),

            authenticated: false,
            auth_fields: AuthFields::default(),
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
        // Show connection profiles if opened instead of auth window
        if self.connection_profiles_component.is_opened() {
            self.connection_profiles_component
                .show(ui, ctx, &mut self.auth_fields);
            return;
        }

        let window_height = ui.available_size().y;

        ui.columns(3, |columns| {
            const MAIN_COLUMN: usize = 1;
            const RIGHT_COLUMN: usize = 2;

            columns[MAIN_COLUMN].vertical_centered(|ui| {
                ui.add_space(window_height / 6.0);

                ui.vertical_centered_justified(|ui| {
                    ui.label(RichText::new(t!("Component.Auth.Login")).size(26.0));
                });

                ui.add_space(window_height / 6.0);

                Grid::new("AuthenticationFields")
                    .num_columns(2)
                    .striped(false)
                    .spacing([20.0, 20.0])
                    .show(ui, |ui| {
                        ui.label(format!("{}:", t!("Component.Auth.IP")));
                        ui.add(
                            TextEdit::singleline(&mut self.auth_fields.ip)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.label(format!("{}:", t!("Component.Auth.Port")));
                        ui.add(
                            TextEdit::singleline(&mut self.auth_fields.port)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();

                        ui.label(format!("{}:", t!("Component.Auth.Password")));
                        ui.add(
                            TextEdit::singleline(&mut self.auth_fields.password)
                                .password(true)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();
                    });

                ui.add_space(window_height / 6.0);

                ui.vertical_centered_justified(|ui| {
                    if ui.button(t!("Component.Auth.Connect")).clicked() {
                        match self.auth_fields.get_address() {
                            Ok(address) => {
                                self.try_connect(
                                    ctx,
                                    address,
                                    &self.auth_fields.password.clone(),
                                );
                            },
                            Err(err) => {
                                let modal = MessageModal::error(&err.localize());
                                let _ = ctx.modals_tx.try_send(Box::new(modal));
                            },
                        }
                    }
                });
            });

            columns[RIGHT_COLUMN].with_layout(
                egui::Layout::right_to_left(egui::Align::Min),
                |ui| {
                    if ui
                        .button("⚙")
                        .on_hover_text(t!("Component.Auth.Hover.ClientSettings"))
                        .clicked()
                    {
                        self.pre_auth_settings_component.open();
                    }
                    if ui
                        .button("☎")
                        .on_hover_text(t!("Component.Auth.Hover.ConnectionProfiles"))
                        .clicked()
                    {
                        self.connection_profiles_component.open();
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
                let _ = ctx.modals_tx.try_send(Box::new(modal));
            },
        }
    }

    pub fn logout(&mut self, ctx: &Context) {
        self.authenticated = false;
        self.net_thread = None;
        self.pre_auth_settings_component.update_tab(ctx);
    }
}

#[derive(Default)]
pub struct AuthFields {
    pub ip: String,
    pub port: String,
    pub password: String,
}

impl AuthFields {
    fn get_address(&self) -> Result<SocketAddr, AuthFieldError> {
        let ip_address: IpAddr = self
            .ip
            .trim()
            .parse()
            .map_err(|_| AuthFieldError::WrongIpAddress)?;
        let port: u16 = self
            .port
            .trim()
            .parse()
            .map_err(|_| AuthFieldError::WrongPort)?;

        Ok(SocketAddr::new(ip_address, port))
    }

    pub fn into_profile(self, title: &str) -> Result<Profile, AuthFieldError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(AuthFieldError::TitleTooShort);
        }

        let password = self.password.trim();
        if password.len() < MIN_PASSWORD_LEN {
            return Err(AuthFieldError::PasswordTooSmall);
        } else if password.len() > MAX_PASSWORD_LEN {
            return Err(AuthFieldError::PasswordTooLarge);
        };

        let profile = Profile {
            title: title.to_string(),
            ip: self
                .ip
                .trim()
                .parse()
                .map_err(|_| AuthFieldError::WrongIpAddress)?,
            port: self
                .port
                .trim()
                .parse()
                .map_err(|_| AuthFieldError::WrongPort)?,
            password: password.to_string(),
        };

        Ok(profile)
    }
}

const MIN_PASSWORD_LEN: usize = 4;
const MAX_PASSWORD_LEN: usize = 20;

#[derive(Debug)]
pub enum AuthFieldError {
    PasswordTooLarge,
    PasswordTooSmall,
    TitleTooShort,
    WrongIpAddress,
    WrongPort,
}

impl AuthFieldError {
    pub fn localize(&self) -> String {
        match self {
            AuthFieldError::TitleTooShort => {
                t!("Component.Auth.Error.TitleTooShort").to_string()
            },
            AuthFieldError::WrongIpAddress => {
                t!("Component.Auth.Error.WrongIpAddress").to_string()
            },
            AuthFieldError::WrongPort => t!("Component.Auth.Error.WrongPort").to_string(),
            AuthFieldError::PasswordTooSmall => {
                t!("Component.Auth.Error.PasswordTooSmall").to_string()
            },
            AuthFieldError::PasswordTooLarge => {
                t!("Component.Auth.Error.PasswordTooLarge").to_string()
            },
        }
    }
}
