use crate::context::Context;
use std::sync::atomic::AtomicBool;
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
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Failed to lock the context")]
    ContextLockError,
}
