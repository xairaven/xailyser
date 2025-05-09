use crate::context::ClientSettings;
use crate::ws::request::UiClientRequest;
use chrono::{DateTime, Local};
use crossbeam::channel::Sender;

pub const DEFAULT_PING_DELAY_SECONDS: i64 = 5;
pub const PING_TIMEOUT_SECONDS: i64 = 5;

#[derive(Default)]
pub struct Heartbeat {
    pub last_sync: Option<DateTime<Local>>,
    ping_sent: bool,
}

impl Heartbeat {
    pub fn check(
        &mut self, client_settings: &ClientSettings, tx: &Sender<UiClientRequest>,
    ) {
        if self.is_ping_needed(client_settings) {
            self.try_ping(tx);
        }
    }

    pub fn try_ping(&mut self, tx: &Sender<UiClientRequest>) {
        if let Err(err) = tx.try_send(UiClientRequest::Ping) {
            log::error!("Failed to send command (Ping): {}", err);
        } else {
            self.ping_sent = true;
        }
    }

    fn is_ping_needed(&self, client_settings: &ClientSettings) -> bool {
        if let Some(last_sync) = &self.last_sync {
            return (Local::now() - last_sync).num_seconds()
                > client_settings.sync_delay_seconds
                && !self.ping_sent;
        }
        false
    }

    pub fn is_timeout(&self, client_settings: &ClientSettings) -> bool {
        if let Some(last_sync) = &self.last_sync {
            let timeout = client_settings.sync_delay_seconds + PING_TIMEOUT_SECONDS;
            return (Local::now() - last_sync).num_seconds() > timeout && self.ping_sent;
        }
        false
    }

    pub fn update(&mut self) {
        self.last_sync = Some(Local::now());
        self.ping_sent = false;
    }
}
