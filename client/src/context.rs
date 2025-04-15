use crate::communication::heartbeat::Heartbeat;
use crate::communication::request::UiClientRequest;
use crate::config::Config;
use crate::ui::modals::Modal;
use crate::ui::themes::ThemePreference;
use chrono::{DateTime, Local};
use common::messages::Response;
use crossbeam::channel::{Receiver, Sender, unbounded};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct Context {
    // Runtime context
    pub client_settings: ClientSettings,
    pub settings_server: ServerSettings,
    pub heartbeat: Heartbeat,

    // Used for saving into config file
    pub config: Config,

    // Shutdown flag
    pub shutdown_flag: Arc<AtomicBool>,

    // Channels
    pub modals_tx: Sender<Box<dyn Modal>>,
    pub modals_rx: Receiver<Box<dyn Modal>>,
    pub server_response_tx: Sender<Response>,
    pub server_response_rx: Receiver<Response>,
    pub ui_client_requests_tx: Sender<UiClientRequest>,
    pub ui_client_requests_rx: Receiver<UiClientRequest>,
}

impl Context {
    pub fn new(config: Config) -> Self {
        let (modals_tx, modals_rx) = unbounded::<Box<dyn Modal>>();
        let (server_response_tx, server_response_rx) = unbounded::<Response>();
        let (ui_client_requests_tx, ui_client_requests_rx) =
            unbounded::<UiClientRequest>();

        Self {
            client_settings: ClientSettings {
                theme: config.theme,
                sync_delay_seconds: config.sync_delay_seconds,
            },
            settings_server: Default::default(),
            heartbeat: Default::default(),

            config,

            shutdown_flag: Arc::new(Default::default()),

            modals_tx,
            modals_rx,
            server_response_tx,
            server_response_rx,
            ui_client_requests_tx,
            ui_client_requests_rx,
        }
    }
}

#[derive(Default)]
pub struct ServerSettings {
    pub interfaces_available: Vec<String>,
    pub interface_active: Option<String>,
    pub interface_active_config: Option<String>,
    pub interfaces_last_updated: Option<DateTime<Local>>,
}

pub struct ClientSettings {
    pub theme: ThemePreference,
    pub sync_delay_seconds: i64,
}
