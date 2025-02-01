use crate::config::Config;
use std::time::Duration;

pub struct Context {
    pub config: Config,
}

pub const RETRY_LOCK_DELAY: Duration = Duration::from_millis(5);
pub const MAX_LOCK_RETRY_ATTEMPTS: usize = 5;
