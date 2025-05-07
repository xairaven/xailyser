use crate::context;
use crate::context::Context;
use crate::net::interface::InterfaceError;
use common::channel::{BroadcastChannel, BroadcastPool};
use dpi::dto::frame::FrameType;
use dpi::parser::ProtocolParser;
use pcap::{Active, Capture};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use thiserror::Error;

const TIMEOUT_MS: i32 = 10;

pub struct PacketSniffer {
    capture: Capture<Active>,
    frame_channel: BroadcastChannel<FrameType>,
    frame_channels_pool: Arc<RwLock<BroadcastPool<FrameType>>>,
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
                self.synchronize_frame_senders();
                match self.capture.next_packet() {
                    Ok(packet) => {
                        let frame = match self.parser.process(packet) {
                            Some(frame) => frame,
                            None => continue,
                        };
                        self.frame_channel.send(frame);
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

    fn synchronize_frame_senders(&mut self) {
        let mut sender_ready = false;
        if let Ok(frame_pool) = self.frame_channels_pool.try_read() {
            if frame_pool.is_sender_ready() {
                sender_ready = true;
            }
        }
        if sender_ready {
            if let Ok(mut value) = self.frame_channels_pool.try_write() {
                while let Some(sender) = value.last_sender() {
                    self.frame_channel.add_sender(sender);
                }
            }
        }
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
}

pub struct PacketSnifferBuilder {
    pub frame_channels_pool: Arc<RwLock<BroadcastPool<FrameType>>>,
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
            frame_channel: BroadcastChannel::<FrameType>::new(),
            frame_channels_pool: self.frame_channels_pool,
            parser,
            shutdown_flag: self.shutdown_flag,
            ws_active_counter: self.ws_active_counter,
        };
        Ok(sniffer)
    }
}
