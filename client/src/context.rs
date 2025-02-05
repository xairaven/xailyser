use crate::commands::UiCommand;
use crate::ui::modals::Modal;
use crate::ui::themes::ThemePreference;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use xailyser_common::messages::ServerResponse;

pub struct Context {
    pub active_theme: ThemePreference,
    pub shutdown_flag: Arc<AtomicBool>,

    pub modals_tx: Sender<Box<dyn Modal>>,
    pub modals_rx: Receiver<Box<dyn Modal>>,
    pub server_response_tx: Sender<ServerResponse>,
    pub server_response_rx: Receiver<ServerResponse>,
    pub ui_commands_tx: Sender<UiCommand>,
    pub ui_commands_rx: Receiver<UiCommand>,
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
        let (server_response_tx, server_response_rx) = unbounded::<ServerResponse>();
        let (ui_commands_tx, ui_commands_rx) = unbounded::<UiCommand>();

        Self {
            active_theme: ThemePreference::default(),
            shutdown_flag: Arc::new(Default::default()),

            modals_tx,
            modals_rx,
            server_response_tx,
            server_response_rx,
            ui_commands_tx,
            ui_commands_rx,
        }
    }
}
