use crate::ui::themes::ThemePreference;
use crate::ui::windows::Window;
use crossbeam::channel::{unbounded, Receiver, Sender};
use xailyser_common::messages::{ClientRequest, ServerResponse};

pub struct Context {
    pub active_theme: ThemePreference,

    pub windows_tx: Sender<Box<dyn Window>>,
    pub windows_rx: Receiver<Box<dyn Window>>,
    pub ws_tx: Sender<ServerResponse>,
    pub ws_rx: Receiver<ServerResponse>,
    pub ui_tx: Sender<ClientRequest>,
    pub ui_rx: Receiver<ClientRequest>,
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
        let (windows_tx, windows_rx) = unbounded::<Box<dyn Window>>();
        let (ws_tx, ws_rx) = unbounded::<ServerResponse>();
        let (ui_tx, ui_rx) = unbounded::<ClientRequest>();

        Self {
            active_theme: ThemePreference::default(),

            windows_tx,
            windows_rx,
            ws_tx,
            ws_rx,
            ui_tx,
            ui_rx,
        }
    }
}
