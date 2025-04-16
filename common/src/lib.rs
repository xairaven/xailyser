// Library lints
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unsafe_code)]

pub mod auth;
pub mod channel;
pub mod compression;
pub mod cryptography;
pub mod logging;
pub mod messages;
