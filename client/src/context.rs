use crate::commands::UiClientRequest;
use crate::ui::modals::Modal;
use crate::ui::themes::ThemePreference;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use xailyser_common::messages::Response;

pub struct Context {
    pub interfaces_available: Vec<String>,
    pub interface_active: Option<String>,

    pub active_theme: ThemePreference,
    pub shutdown_flag: Arc<AtomicBool>,

    pub modals_tx: Sender<Box<dyn Modal>>,
    pub modals_rx: Receiver<Box<dyn Modal>>,
    pub server_response_tx: Sender<Response>,
    pub server_response_rx: Receiver<Response>,
    pub ui_client_requests_tx: Sender<UiClientRequest>,
    pub ui_client_requests_rx: Receiver<UiClientRequest>,
}

impl Context {
    pub fn new(theme: ThemePreference) -> Self {
        Self {
            active_theme: theme,

            ..Default::default()
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        let (modals_tx, modals_rx) = unbounded::<Box<dyn Modal>>();
        let (server_response_tx, server_response_rx) = unbounded::<Response>();
        let (ui_client_requests_tx, ui_client_requests_rx) =
            unbounded::<UiClientRequest>();

        Self {
            interfaces_available: vec![],
            interface_active: None,

            active_theme: ThemePreference::default(),
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
