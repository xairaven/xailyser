use crate::communication::heartbeat::Heartbeat;
use crate::communication::request::UiClientRequest;
use crate::config::Config;
use crate::net::NetStorage;
use crate::net::device::DeviceStorage;
use crate::net::lookup::Lookup;
use crate::net::raw::RawStorage;
use crate::net::speed::PlotSettings;
use crate::profiles::ProfilesStorage;
use crate::ui::modals::Modal;
use crate::ui::styles::themes;
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
    pub net_storage: NetStorage,

    // Used for saving into config file
    pub config: Config,
    // Connection profiles
    pub profiles_storage: ProfilesStorage,

    // Shutdown flag
    pub shutdown_flag: Arc<AtomicBool>,

    // Channels
    pub modals_tx: Sender<Box<dyn Modal>>,
    pub modals_rx: Receiver<Box<dyn Modal>>,
    pub data_response_tx: Sender<Response>,
    pub data_response_rx: Receiver<Response>,
    pub server_response_tx: Sender<Response>,
    pub server_response_rx: Receiver<Response>,
    pub ui_client_requests_tx: Sender<UiClientRequest>,
    pub ui_client_requests_rx: Receiver<UiClientRequest>,
}

impl Context {
    pub fn new(config: Config) -> Self {
        let (modals_tx, modals_rx) = unbounded::<Box<dyn Modal>>();
        let (server_response_tx, server_response_rx) = unbounded::<Response>();
        let (data_response_tx, data_response_rx) = unbounded::<Response>();
        let (ui_client_requests_tx, ui_client_requests_rx) =
            unbounded::<UiClientRequest>();

        let profiles_storage = ProfilesStorage::from_file().unwrap_or_default();
        let lookup = match Lookup::load() {
            Ok(lookup) => lookup,
            Err(err) => {
                log::error!("Failed to load lookup database: {}", err);
                std::process::exit(1);
            },
        };

        Self {
            client_settings: ClientSettings {
                compression: config.compression,
                plot: PlotSettings {
                    display_window_seconds: config.plot_display_window_seconds,
                    units: config.plot_speed_units.clone(),
                },
                sync_delay_seconds: config.sync_delay_seconds,
                theme: config.theme,
                unparsed_frames_drop: config.unparsed_frames_drop,
                unparsed_frames_threshold: config.unparsed_frames_threshold,
            },
            settings_server: Default::default(),
            heartbeat: Default::default(),
            net_storage: NetStorage {
                devices: DeviceStorage::default(),
                lookup,
                raw: RawStorage::new(config.unparsed_frames_threshold),
                speed: Default::default(),
            },

            config,
            profiles_storage,

            shutdown_flag: Arc::new(Default::default()),

            modals_tx,
            modals_rx,
            data_response_tx,
            data_response_rx,
            server_response_tx,
            server_response_rx,
            ui_client_requests_tx,
            ui_client_requests_rx,
        }
    }

    pub fn logout(&mut self) {
        // Cloning client settings, config, lookup, profiles storage.

        let (modals_tx, modals_rx) = unbounded::<Box<dyn Modal>>();
        let (server_response_tx, server_response_rx) = unbounded::<Response>();
        let (data_response_tx, data_response_rx) = unbounded::<Response>();
        let (ui_client_requests_tx, ui_client_requests_rx) =
            unbounded::<UiClientRequest>();

        let profiles_storage = self.profiles_storage.clone();

        let new_context = Self {
            client_settings: self.client_settings.clone(),
            settings_server: Default::default(),
            heartbeat: Default::default(),
            net_storage: NetStorage {
                devices: DeviceStorage::default(),
                lookup: self.net_storage.lookup.clone(),
                raw: RawStorage::new(self.config.unparsed_frames_threshold),
                speed: Default::default(),
            },

            config: self.config.clone(),
            profiles_storage,

            shutdown_flag: Arc::new(Default::default()),

            modals_tx,
            modals_rx,
            data_response_tx,
            data_response_rx,
            server_response_tx,
            server_response_rx,
            ui_client_requests_tx,
            ui_client_requests_rx,
        };

        *self = new_context;
    }
}

#[derive(Default)]
pub struct ServerSettings {
    pub compression_active: bool,
    pub compression_config: bool,
    pub interfaces_available: Vec<String>,
    pub interface_active: Option<String>,
    pub interface_config: Option<String>,
    pub link_type: Option<pcap::Linktype>,
    pub send_unparsed_frames_active: bool,
    pub send_unparsed_frames_config: bool,

    pub last_updated: Option<DateTime<Local>>,
}

#[derive(Clone)]
pub struct ClientSettings {
    pub compression: bool,
    pub sync_delay_seconds: i64,
    pub plot: PlotSettings,
    pub theme: themes::Preference,
    pub unparsed_frames_drop: bool,
    pub unparsed_frames_threshold: Option<usize>,
}
