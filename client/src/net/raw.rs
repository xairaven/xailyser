use std::collections::VecDeque;
use std::path::PathBuf;

pub struct RawStorage {
    metadata: VecDeque<dpi::wrapper::PacketHeader>,
    frames: VecDeque<dpi::wrapper::OwnedPacket>,

    threshold: Option<usize>,
}

impl RawStorage {
    pub fn new(threshold: Option<usize>) -> Self {
        Self {
            metadata: Default::default(),
            frames: Default::default(),
            threshold,
        }
    }

    pub fn add_frame(&mut self, frame: dpi::wrapper::OwnedPacket) {
        self.frames.push_back(frame);
        if let Some(threshold) = self.threshold {
            if self.frames.len() > threshold {
                self.frames.pop_front();
            }
        }
    }

    pub fn add_metadata(&mut self, frame: dpi::wrapper::PacketHeader) {
        self.metadata.push_back(frame);
        if let Some(threshold) = self.threshold {
            if self.metadata.len() > threshold {
                self.metadata.pop_front();
            }
        }
    }

    pub fn set_threshold(&mut self, threshold: Option<usize>) {
        self.threshold = threshold;
    }

    pub fn frames_amount(&self) -> usize {
        self.frames.len()
    }

    pub fn metadata_amount(&self) -> usize {
        self.metadata.len()
    }

    pub fn clear_metadata(&mut self) {
        self.metadata.clear();
    }

    pub fn clear_frames(&mut self) {
        self.frames.clear();
    }

    pub fn is_frames_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn is_metadata_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    pub fn save_pcap(
        &mut self, path: PathBuf, link_type: pcap::Linktype,
    ) -> Result<(), pcap::Error> {
        let result = dpi::wrapper::save_pcap(path, &self.frames, link_type);
        if result.is_ok() {
            self.frames.clear();
        }

        result
    }
}
