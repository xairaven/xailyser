// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::indexing_slicing)]
#![deny(unsafe_code)]

pub mod analysis {
    pub mod ports;
    pub mod vendor;
}
pub mod dto {
    pub mod frame;
    pub mod metadata;
}
pub mod parser;
pub mod protocols;

// TODO tag: Needed features/fixes for release
// FUTURE tag: Features, that may be implemented in future
