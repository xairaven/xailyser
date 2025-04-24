use std::path::PathBuf;

#[derive(Default)]
pub struct RawStorage {
    metadata: Vec<dpi::wrapper::PacketHeader>,
    frames: Vec<dpi::wrapper::OwnedPacket>,
}

impl RawStorage {
    pub fn add_frame(&mut self, frame: dpi::wrapper::OwnedPacket) {
        self.frames.push(frame);
    }

    pub fn add_metadata(&mut self, frame: dpi::wrapper::PacketHeader) {
        self.metadata.push(frame);
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
