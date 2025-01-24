use crate::ui::windows::Window;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::net::SocketAddr;

pub struct Context {
    pub address: Option<SocketAddr>,

    pub windows_tx: Sender<Box<dyn Window>>,
    pub windows_rx: Receiver<Box<dyn Window>>,
}

impl Default for Context {
    fn default() -> Self {
        let (windows_tx, windows_rx) = unbounded::<Box<dyn Window>>();

        Self {
            address: None,

            windows_tx,
            windows_rx,
        }
    }
}
