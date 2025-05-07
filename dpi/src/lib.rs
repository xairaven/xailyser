// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::indexing_slicing)]
#![deny(unsafe_code)]

pub mod dto {
    pub mod frame;
}
pub mod parser;
pub mod protocols;

// TODO tag: Needed features/fixes for release
// FUTURE tag: Features, that may be implemented in future
