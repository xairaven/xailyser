use crate::metadata::FrameMetadata;
use crate::protocols::Protocol;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ethernet;

impl Protocol for Ethernet {
    fn parse(bytes: &[u8], metadata: &mut FrameMetadata) -> bool {
        todo!()
    }
}
