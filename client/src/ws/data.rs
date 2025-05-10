use crate::context::Context;
use crate::net::device::LocalDeviceBuilder;
use crate::net::speed::{Sample, SampleDirection, SpeedError};
use dpi::dto::frame::{FrameHeader, OwnedFrame};
use dpi::dto::metadata::{FrameMetadataDto, ProtocolDto};
use dpi::protocols::ethernet::mac::MacAddress;
use std::net::{Ipv4Addr, Ipv6Addr};
use thiserror::Error;

pub fn metadata(
    ctx: &mut Context, metadata: FrameMetadataDto,
) -> Result<(), ProcessingError> {
    let mut sample = Some(Sample::try_from(&metadata.header)?);

    if metadata.layers.is_empty() {
        return header(ctx, metadata.header);
    }

    let datalink_info = match metadata.layers.first() {
        Some(ProtocolDto::Ethernet(ethernet_info)) => ethernet_info.clone(),
        _ => return Err(ProcessingError::DatalinkNotFirst),
    };

    let mut locator = Locator {
        mac: (
            datalink_info.source_mac.clone(),
            datalink_info.destination_mac.clone(),
        ),
        ipv4: None,
        ipv6: None,
    };

    let limit = &ctx.client_settings.parsed_frames_limit;
    let frames_len = &ctx.net_storage.inspector.ethernet.len();

    let mut device_builder: Option<LocalDeviceBuilder> = None;
    for layer in metadata.layers.into_iter().skip(1) {
        match layer {
            ProtocolDto::Ethernet(_) => return Err(ProcessingError::DatalinkNotFirst),
            ProtocolDto::Arp(value) => {
                push_value(&mut ctx.net_storage.inspector.arp, value, limit, frames_len)
            },
            ProtocolDto::DHCPv4(value) => push_value(
                &mut ctx.net_storage.inspector.dhcpv4,
                value,
                limit,
                frames_len,
            ),
            ProtocolDto::DHCPv6(value) => push_value(
                &mut ctx.net_storage.inspector.dhcpv6,
                value,
                limit,
                frames_len,
            ),
            ProtocolDto::DNS(value) => {
                push_value(&mut ctx.net_storage.inspector.dns, value, limit, frames_len)
            },
            ProtocolDto::HTTP(value) => push_value(
                &mut ctx.net_storage.inspector.http,
                (value, locator.clone()),
                limit,
                frames_len,
            ),
            ProtocolDto::IPv4(ipv4) => {
                if ipv4.address_source.is_private() {
                    if let Some(sample) = sample.take() {
                        ctx.net_storage
                            .speed
                            .load_complete_sample(SampleDirection::Send(sample));
                    }
                    let mut builder =
                        LocalDeviceBuilder::new(datalink_info.source_mac.clone());
                    builder.ip.push(ipv4.address_source);
                    device_builder = Some(builder);
                }
                if ipv4.address_destination.is_private() {
                    if let Some(sample) = sample.take() {
                        ctx.net_storage
                            .speed
                            .load_complete_sample(SampleDirection::Receive(sample));
                    }
                    let mut builder =
                        LocalDeviceBuilder::new(datalink_info.destination_mac.clone());
                    builder.ip.push(ipv4.address_destination);
                    device_builder = Some(builder);
                }
                locator.ipv4 = Some((ipv4.address_source, ipv4.address_destination));
                push_value(
                    &mut ctx.net_storage.inspector.ipv4,
                    (ipv4, locator.clone()),
                    limit,
                    frames_len,
                );
            },
            ProtocolDto::IPv6(ipv6) => {
                if ipv6.address_source.is_unique_local() {
                    if let Some(sample) = sample.take() {
                        ctx.net_storage
                            .speed
                            .load_complete_sample(SampleDirection::Send(sample));
                    }
                    let mut builder =
                        LocalDeviceBuilder::new(datalink_info.source_mac.clone());
                    builder.ipv6.push(ipv6.address_source);
                    device_builder = Some(builder);
                }
                if ipv6.address_destination.is_unique_local() {
                    if let Some(sample) = sample.take() {
                        ctx.net_storage
                            .speed
                            .load_complete_sample(SampleDirection::Receive(sample));
                    }
                    let mut builder =
                        LocalDeviceBuilder::new(datalink_info.destination_mac.clone());
                    builder.ipv6.push(ipv6.address_destination);
                    device_builder = Some(builder);
                }
                locator.ipv6 = Some((ipv6.address_source, ipv6.address_destination));
                push_value(
                    &mut ctx.net_storage.inspector.ipv6,
                    (ipv6, locator.clone()),
                    limit,
                    frames_len,
                );
            },
            ProtocolDto::ICMPv4(value) => push_value(
                &mut ctx.net_storage.inspector.icmpv4,
                (value, locator.clone()),
                limit,
                frames_len,
            ),
            ProtocolDto::ICMPv6(value) => push_value(
                &mut ctx.net_storage.inspector.icmpv6,
                (value, locator.clone()),
                limit,
                frames_len,
            ),
            ProtocolDto::TCP(value) => push_value(
                &mut ctx.net_storage.inspector.tcp,
                (value, locator.clone()),
                limit,
                frames_len,
            ),
            ProtocolDto::UDP(value) => push_value(
                &mut ctx.net_storage.inspector.udp,
                (value, locator.clone()),
                limit,
                frames_len,
            ),
        }
    }

    // Pushing ethernet
    push_value(
        &mut ctx.net_storage.inspector.ethernet,
        (datalink_info, locator),
        limit,
        frames_len,
    );

    // Pushing sample to speed plot (not pushed as sent or received yet)
    if let Some(sample) = sample {
        ctx.net_storage
            .speed
            .load_complete_sample(SampleDirection::Throughput(sample));
    }

    // Adding info if device exists, adding device if not
    if let Some(mut builder) = device_builder {
        if let Some(device) = ctx.net_storage.devices.find_by_mac(&builder.mac) {
            device.ip_mut().append(&mut builder.ip);
            device.ipv6_mut().append(&mut builder.ipv6);
        } else {
            builder.vendor = ctx.net_storage.lookup.find_vendor(&builder.mac);
            ctx.net_storage.devices.add_device(builder);
        }
    }

    Ok(())
}

pub fn header(ctx: &mut Context, header: FrameHeader) -> Result<(), ProcessingError> {
    let sample = Sample::try_from(&header)?;
    ctx.net_storage.speed.load_raw_sample(sample);

    Ok(())
}

pub fn raw(ctx: &mut Context, raw: OwnedFrame) -> Result<(), ProcessingError> {
    let sample = Sample::try_from(&raw.header)?;
    ctx.net_storage.speed.load_raw_sample(sample);

    if !ctx.client_settings.unparsed_frames_drop {
        ctx.net_storage.raw.add(raw);
    }
    // Else - pass

    Ok(())
}

#[derive(Clone, Debug)]
pub struct Locator {
    pub mac: (MacAddress, MacAddress),
    pub ipv4: Option<(Ipv4Addr, Ipv4Addr)>,
    pub ipv6: Option<(Ipv6Addr, Ipv6Addr)>,
}

impl Locator {
    pub fn ip_to_string(&self) -> (String, String) {
        let (source_ip, target_ip) = match self.ipv4 {
            Some(addresses) => (addresses.0.to_string(), addresses.1.to_string()),
            None => match self.ipv6 {
                Some(addresses) => (addresses.0.to_string(), addresses.1.to_string()),
                None => ("-".to_string(), "-".to_string()),
            },
        };
        (source_ip, target_ip)
    }
}

fn push_value<T>(vec: &mut Vec<T>, value: T, limit: &Option<usize>, frames_len: &usize) {
    if let Some(limit) = limit {
        if frames_len < limit {
            vec.push(value);
        }
    } else {
        vec.push(value);
    }
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Speed Error.")]
    Speed(#[from] SpeedError),

    #[error("Empty layers packet got to full metadata processing.")]
    DatalinkNotFirst,
}
