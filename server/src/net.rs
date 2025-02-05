use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct PacketSniffer {
    shutdown_flag: Arc<AtomicBool>,
}

// TODO
impl PacketSniffer {
    pub fn new(shutdown_flag: Arc<AtomicBool>) -> Self {
        Self { shutdown_flag }
    }

    pub fn start(&self) {
        // while !self.shutdown_flag.load(Ordering::Acquire) {
        //
        // }
    }
}

pub mod interface;
