use crate::ui::windows::Window;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::{LazyLock, Mutex};

pub static CONTEXT: LazyLock<Mutex<Context>> =
    LazyLock::new(|| Mutex::new(Context::default()));

pub struct Context {
    pub windows_tx: Sender<Box<dyn Window>>,
    pub windows_rx: Receiver<Box<dyn Window>>,
}

impl Default for Context {
    fn default() -> Self {
        let (windows_tx, windows_rx) = unbounded::<Box<dyn Window>>();

        Self {
            windows_tx,
            windows_rx,
        }
    }
}
