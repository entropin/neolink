use serde::Deserialize;
use std::net::SocketAddr;

***REMOVED***[derive(Debug, Deserialize)]
pub struct Config {
    pub cameras: Vec<CameraConfig>,

    ***REMOVED***[serde(rename="bind", default="default_bind_addr")]
    pub bind_addr: String,
}

***REMOVED***[derive(Debug, Deserialize)]
pub struct CameraConfig {
    pub name: String,

    ***REMOVED***[serde(rename="address")]
    pub camera_addr: SocketAddr,

    pub username: String,
    pub password: Option<String>,
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}
