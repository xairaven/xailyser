// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

pub mod frame;
pub mod parser;
pub mod protocols;
pub mod wrapper;
