use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::clone::Clone;
use std::time::Duration;
use validator::{Validate, ValidationError};
use validator_derive::Validate;

lazy_static! {
    static ref RE_STREAM_SRC: Regex = Regex::new(r"^(mainStream|subStream|both)$").unwrap();
    static ref RE_TLS_CLIENT_AUTH: Regex = Regex::new(r"^(none|request|require)$").unwrap();
}

***REMOVED***[derive(Debug, Deserialize, Validate, Clone)]
pub struct Config {
    ***REMOVED***[validate]
    pub cameras: Vec<CameraConfig>,

    ***REMOVED***[serde(rename = "bind", default = "default_bind_addr")]
    pub bind_addr: String,

    ***REMOVED***[validate(range(min = 0, max = 65535, message = "Invalid port", code = "bind_port"))]
    ***REMOVED***[serde(default = "default_bind_port")]
    pub bind_port: u16,

    ***REMOVED***[serde(default = "default_certificate")]
    pub certificate: Option<String>,

    ***REMOVED***[validate(regex(
        path = "RE_TLS_CLIENT_AUTH",
        message = "Incorrect tls auth",
        code = "tls_client_auth"
    ))]
    ***REMOVED***[serde(default = "default_tls_client_auth")]
    pub tls_client_auth: String,

    ***REMOVED***[validate]
    ***REMOVED***[serde(default)]
    pub users: Vec<UserConfig>,
}

***REMOVED***[derive(Debug, Deserialize, Validate, Clone)]
pub struct CameraConfig {
    pub name: String,

    ***REMOVED***[serde(rename = "address")]
    pub camera_addr: String,

    pub username: String,
    pub password: Option<String>,

    // no longer used, but still here so we can warn users:
    pub timeout: Option<Duration>,

    // no longer used, but still here so we can warn users:
    pub format: Option<String>,

    ***REMOVED***[validate(regex(
        path = "RE_STREAM_SRC",
        message = "Incorrect stream source",
        code = "stream"
    ))]
    ***REMOVED***[serde(default = "default_stream")]
    pub stream: String,

    pub permitted_users: Option<Vec<String>>,

    ***REMOVED***[validate(range(min = 0, max = 31, message = "Invalid channel", code = "channel_id"))]
    ***REMOVED***[serde(default = "default_channel_id")]
    pub channel_id: u8,
}

***REMOVED***[derive(Debug, Deserialize, Validate, Clone)]
pub struct UserConfig {
    ***REMOVED***[validate(custom = "validate_username")]
    ***REMOVED***[serde(alias = "username")]
    pub name: String,

    ***REMOVED***[serde(alias = "password")]
    pub pass: String,
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}

fn default_bind_port() -> u16 {
    8554
}

fn default_stream() -> String {
    "both".to_string()
}

fn default_certificate() -> Option<String> {
    None
}

fn default_tls_client_auth() -> String {
    "none".to_string()
}

fn default_channel_id() -> u8 {
    0
}

pub static RESERVED_NAMES: &[&str] = &["anyone", "anonymous"];
fn validate_username(name: &str) -> Result<(), ValidationError> {
    if name.trim().is_empty() {
        return Err(ValidationError::new("username cannot be empty"));
    }
    if RESERVED_NAMES.contains(&name) {
        return Err(ValidationError::new("This is a reserved username"));
    }
    Ok(())
}
