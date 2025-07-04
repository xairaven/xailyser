use crate::context::ClientSettings;
use chrono::{DateTime, Local, TimeZone};
use dpi::dto::frame::FrameHeader;
use std::collections::VecDeque;
use std::fmt::Formatter;
use strum_macros::EnumIter;
use thiserror::Error;

#[derive(Default)]
pub struct SpeedData {
    throughput: VecDeque<Sample>,
    send: VecDeque<Sample>,
    receive: VecDeque<Sample>,

    bucket_throughput: Vec<f64>,
    bucket_send: Vec<f64>,
    bucket_receive: Vec<f64>,
}

impl SpeedData {
    pub fn load_complete_sample(&mut self, directed_sample: SampleDirection) {
        match directed_sample {
            SampleDirection::Send(value) => {
                self.throughput.push_back(value.clone());
                self.send.push_back(value)
            },
            SampleDirection::Receive(value) => {
                self.throughput.push_back(value.clone());
                self.receive.push_back(value)
            },
            SampleDirection::Throughput(value) => self.throughput.push_back(value),
        }
    }

    pub fn load_raw_sample(&mut self, sample: Sample) {
        self.throughput.push_back(sample);
    }

    pub fn update_info(&mut self, settings: &ClientSettings) {
        let now = Local::now();

        Self::clear_deque_outdated(&mut self.throughput, settings, now);
        Self::clear_deque_outdated(&mut self.send, settings, now);
        Self::clear_deque_outdated(&mut self.receive, settings, now);

        Self::bucket_per_second(
            &mut self.bucket_throughput,
            &self.throughput,
            &settings.plot,
            now,
        );
        Self::bucket_per_second(&mut self.bucket_send, &self.send, &settings.plot, now);
        Self::bucket_per_second(
            &mut self.bucket_receive,
            &self.receive,
            &settings.plot,
            now,
        );
    }

    fn clear_deque_outdated(
        deque: &mut VecDeque<Sample>, settings: &ClientSettings, now: DateTime<Local>,
    ) {
        while let Some(sample) = deque.front() {
            if sample.is_outdated(now, settings) {
                deque.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn throughput_iter(&self) -> impl Iterator<Item = [f64; 2]> {
        self.bucket_throughput
            .iter()
            .enumerate()
            .map(|(i, value)| [i as f64, *value])
    }

    pub fn send_iter(&self) -> impl Iterator<Item = [f64; 2]> {
        self.bucket_send
            .iter()
            .enumerate()
            .map(|(i, value)| [i as f64, *value])
    }

    pub fn receive_iter(&self) -> impl Iterator<Item = [f64; 2]> {
        self.bucket_receive
            .iter()
            .enumerate()
            .map(|(i, value)| [i as f64, *value])
    }

    pub fn peak_throughput(&self) -> f64 {
        Self::peak(&self.bucket_throughput)
    }

    pub fn peak_received(&self) -> f64 {
        Self::peak(&self.bucket_receive)
    }

    pub fn peak_sent(&self) -> f64 {
        Self::peak(&self.bucket_send)
    }

    pub fn peak(bucket: &[f64]) -> f64 {
        bucket
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    fn bucket_per_second(
        bucket: &mut Vec<f64>, deque: &VecDeque<Sample>, settings: &PlotSettings,
        now: DateTime<Local>,
    ) {
        let seconds_max = settings.display_window_seconds as usize + 1;
        bucket.clear();
        if bucket.len() != seconds_max {
            bucket.resize(seconds_max, 0.0);
        }

        for sample in deque {
            let second = (now - sample.time_captured).num_seconds();
            let second = match usize::try_from(second) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if second < seconds_max {
                bucket[second] += settings.units.value(sample.captured_bytes as f64);
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct PlotSettings {
    pub display_window_seconds: u32,
    pub units: SpeedUnitPerSecond,
}

#[derive(Debug, Clone, EnumIter, PartialEq)]
pub enum SpeedUnitPerSecond {
    Bits,
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
}

pub const BITS_PER_SECOND: &str = "bit/Ss";
pub const BYTES_PER_SECOND: &str = "b/s";
pub const KILOBYTES_PER_SECOND: &str = "kB/s";
pub const MEGABYTES_PER_SECOND: &str = "MB/s";
pub const GIGABYTES_PER_SECOND: &str = "GB/s";

const BIT_MULTIPLIER: f64 = 8.0;
const KILOBYTE_DIVIDER: f64 = 1024.0;
const MEGABYTE_DIVIDER: f64 = 1024.0 * 1024.0;
const GIGABYTE_DIVIDER: f64 = 1024.0 * 1024.0 * 1024.0;

impl SpeedUnitPerSecond {
    pub fn value(&self, value: f64) -> f64 {
        match self {
            SpeedUnitPerSecond::Bits => value * BIT_MULTIPLIER,
            SpeedUnitPerSecond::Bytes => value,
            SpeedUnitPerSecond::Kilobytes => value / KILOBYTE_DIVIDER,
            SpeedUnitPerSecond::Megabytes => value / MEGABYTE_DIVIDER,
            SpeedUnitPerSecond::Gigabytes => value / GIGABYTE_DIVIDER,
        }
    }
}

impl std::fmt::Display for SpeedUnitPerSecond {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Bits => BITS_PER_SECOND,
            Self::Bytes => BYTES_PER_SECOND,
            Self::Kilobytes => KILOBYTES_PER_SECOND,
            Self::Megabytes => MEGABYTES_PER_SECOND,
            Self::Gigabytes => GIGABYTES_PER_SECOND,
        };
        write!(f, "{text}")
    }
}

impl TryFrom<&str> for SpeedUnitPerSecond {
    type Error = SpeedError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            BITS_PER_SECOND => Ok(Self::Bits),
            BYTES_PER_SECOND => Ok(Self::Bytes),
            KILOBYTES_PER_SECOND => Ok(Self::Kilobytes),
            MEGABYTES_PER_SECOND => Ok(Self::Megabytes),
            GIGABYTES_PER_SECOND => Ok(Self::Gigabytes),
            _ => Err(Self::Error::FailedToConvertSpeedUnit),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub captured_bytes: u32,
    pub time_captured: DateTime<Local>,
}

pub enum SampleDirection {
    Send(Sample),
    Receive(Sample),
    Throughput(Sample),
}

impl Sample {
    fn is_outdated(&self, now: DateTime<Local>, settings: &ClientSettings) -> bool {
        (now - self.time_captured).num_seconds()
            > settings.plot.display_window_seconds as i64
    }
}

impl TryFrom<&FrameHeader> for Sample {
    type Error = SpeedError;

    fn try_from(header: &FrameHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            captured_bytes: header.caplen,
            time_captured: Local
                .timestamp_opt(header.tv_sec, header.tv_usec as u32)
                .single()
                .ok_or(Self::Error::FailedToConvertCapturedTime)?,
        })
    }
}

#[derive(Error, Debug)]
pub enum SpeedError {
    #[error("Failed to parse captured time.")]
    FailedToConvertCapturedTime,

    #[error("Failed to convert speed unit.")]
    FailedToConvertSpeedUnit,
}
