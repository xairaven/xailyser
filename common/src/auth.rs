pub const AUTH_HEADER: &str = "AUTH-KEY";
pub const COMPRESSION_HEADER: &str = "COMPRESSION-VALUE";

pub mod errors {
    pub const COMPRESSION_HEADER_NOT_FOUND: &str = "Compression header was not found.";
    pub const PASSWORD_HEADER_NOT_FOUND: &str = "Password header was not found.";
    pub const WRONG_COMPRESSION: &str = "Server has other compression settings.";
    pub const WRONG_PASSWORD: &str = "Wrong password.";
}
