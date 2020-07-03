use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;
use std::clone::Clone;
use validator::Validate;
use regex::Regex;

lazy_static! {
    static ref RE_STREAM_FORM: Regex = Regex::new(r"^([hH]26[45]|[ \t]*[!].*)$").unwrap();
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
    pub certificate: String,

    ***REMOVED***[serde(default = "default_username")]
    pub username: String,

    ***REMOVED***[serde(default = "default_password")]
    pub password: String,

    ***REMOVED***[validate(regex(path = "RE_TLS_CLIENT_AUTH", message = "Incorrect stream format", code = "format"))]
    ***REMOVED***[serde(default = "default_tls_client_auth")]
    pub tls_client_auth: String,
}

***REMOVED***[derive(Debug, Deserialize, Validate, Clone)]
pub struct CameraConfig {
    pub name: String,

    ***REMOVED***[serde(rename = "address")]
    pub camera_addr: SocketAddr,

    pub username: String,
    pub password: Option<String>,

    pub timeout: Option<Duration>,

    ***REMOVED***[validate(regex(path = "RE_STREAM_FORM", message = "Incorrect stream format", code = "format"))]
    ***REMOVED***[serde(default = "default_format")]
    pub format: String,

    ***REMOVED***[validate(regex(path = "RE_STREAM_SRC", message = "Incorrect stream source", code = "stream"))]
    ***REMOVED***[serde(default = "default_stream")]
    pub stream: String,
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}

fn default_bind_port() -> u16 {
    8554
}

fn default_format() -> String {
    "h265".to_string()
}

fn default_stream() -> String {
    "both".to_string()
}

fn default_certificate() -> String {
    "".to_string()
}

fn default_tls_client_auth() -> String {
    "none".to_string()
}

fn default_username() -> String {
    "".to_string()
}

fn default_password() -> String {
    "".to_string()
}
