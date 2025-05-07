use dpi::dto::frame::OwnedFrame;
use std::collections::VecDeque;
use std::path::PathBuf;

pub struct RawStorage {
    vec: VecDeque<OwnedFrame>,
    threshold: Option<usize>,
}

impl RawStorage {
    pub fn new(threshold: Option<usize>) -> Self {
        Self {
            vec: Default::default(),
            threshold,
        }
    }

    pub fn add(&mut self, frame: OwnedFrame) {
        self.vec.push_back(frame);
        if let Some(threshold) = self.threshold {
            if self.vec.len() > threshold {
                self.vec.pop_front();
            }
        }
    }

    pub fn set_threshold(&mut self, threshold: Option<usize>) {
        self.threshold = threshold;
    }

    pub fn amount(&self) -> usize {
        self.vec.len()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn save_pcap(
        &mut self, path: PathBuf, link_type: pcap::Linktype,
    ) -> Result<(), pcap::Error> {
        let result = dpi::dto::frame::save_pcap(path, &self.vec, link_type);
        if result.is_ok() {
            self.vec.clear();
        }

        result
    }
}
