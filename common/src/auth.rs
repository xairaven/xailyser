pub const AUTH_HEADER: &str = "AUTH-KEY";

pub mod errors {
    pub const HEADER_NOT_FOUND_ERROR: &str = "Password header was not found.";
    pub const WRONG_PASSWORD_ERROR: &str = "Wrong password.";
}
