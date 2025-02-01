use crate::context::Context;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub struct NetThreadHandler {
    context: Arc<Mutex<Context>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl NetThreadHandler {
    pub fn new(context: Arc<Mutex<Context>>, shutdown_flag: Arc<AtomicBool>) -> Self {
        Self {
            context,
            shutdown_flag,
        }
    }

    pub fn start(&self) -> Result<(), NetError> {
        while !self.shutdown_flag.load(Ordering::Acquire) {
            // Do smth
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Failed to lock the context")]
    ContextLockError,
}
