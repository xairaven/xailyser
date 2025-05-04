use crate::context;
use crate::context::Context;
use crate::net::interface::InterfaceError;
use common::channel::BroadcastChannel;
use dpi::parser::ProtocolParser;
use pcap::{Active, Capture};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thiserror::Error;

const TIMEOUT_MS: i32 = 10;

pub struct PacketSniffer {
    capture: Capture<Active>,
    frame_channel: Arc<Mutex<BroadcastChannel<dpi::frame::FrameType>>>,
    parser: ProtocolParser,
    shutdown_flag: Arc<AtomicBool>,
    ws_active_counter: Arc<AtomicUsize>,
}

impl PacketSniffer {
    pub fn listen(&mut self) -> Result<(), NetworkError> {
        loop {
            if self.shutdown_flag.load(Ordering::Acquire) {
                log::info!("Shutting down net-capturing thread.");
                break;
            }

            if self.ws_active_counter.load(Ordering::Acquire) > 0 {
                match self.capture.next_packet() {
                    Ok(packet) => {
                        let frame = match self.parser.process(packet) {
                            Some(frame) => frame,
                            None => continue,
                        };

                        match self.frame_channel.lock() {
                            Ok(mut guard) => {
                                guard.send(frame);
                            },
                            Err(err) => {
                                log::error!(
                                    "Net: Poison error on frame channel: {}",
                                    err
                                );
                                self.shutdown_flag.store(true, Ordering::Release);
                                return Err(NetworkError::PoisonError(err.to_string()));
                            },
                        }
                    },
                    Err(pcap::Error::TimeoutExpired) => {
                        thread::sleep(Duration::from_millis(TIMEOUT_MS as u64));
                    },
                    Err(err) => {
                        return Err(NetworkError::PcapError(err));
                    },
                }
            } else {
                thread::sleep(Duration::from_millis(TIMEOUT_MS as u64));
            }
        }

        Ok(())
    }
}

pub mod interface;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Interface is absent.")]
    NoInterface,

    #[error("Interface error.")]
    InterfaceError(#[from] InterfaceError),

    #[error("Pcap library error.")]
    PcapError(pcap::Error),

    #[error("Mutex is poisoned.")]
    PoisonError(String),
}

pub struct PacketSnifferBuilder {
    pub frame_channel: Arc<Mutex<BroadcastChannel<dpi::frame::FrameType>>>,
    pub context: Arc<Mutex<Context>>,
    pub shutdown_flag: Arc<AtomicBool>,
    pub ws_active_counter: Arc<AtomicUsize>,
}

impl PacketSnifferBuilder {
    pub fn build(self) -> Result<PacketSniffer, NetworkError> {
        let interface = context::lock(&self.context, |ctx| ctx.network_interface.clone())
            .ok_or(NetworkError::NoInterface)?;
        let capture = interface::get_capture(interface, TIMEOUT_MS)
            .map_err(NetworkError::InterfaceError)?;

        let link_type = capture.get_datalink();
        context::lock(&self.context, |ctx| {
            ctx.link_type = Some(link_type);
        });

        let send_unparsed_frames =
            context::lock(&self.context, |ctx| ctx.send_unparsed_frames);
        let parser = ProtocolParser::new(&link_type, send_unparsed_frames);

        let sniffer = PacketSniffer {
            capture,
            frame_channel: self.frame_channel,
            parser,
            shutdown_flag: self.shutdown_flag,
            ws_active_counter: self.ws_active_counter,
        };
        Ok(sniffer)
    }
}
