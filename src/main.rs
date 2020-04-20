mod bc;
mod bc_protocol;
mod config;

use bc_protocol::BcCamera;
use config::{Config, CameraConfig};
use crossbeam_utils::thread;
use err_derive::Error;
use std::env::args;
use std::fs;
use std::net::TcpListener;

***REMOVED***[derive(Debug, Error)]
pub enum Error {
    ***REMOVED***[error(display="Configuration parsing error")]
    ConfigError(***REMOVED***[error(source)] toml::de::Error),
    ***REMOVED***[error(display="Communication error")]
    ProtocolError(***REMOVED***[error(source)] bc_protocol::Error),
    ***REMOVED***[error(display="I/O error")]
    IoError(***REMOVED***[error(source)] std::io::Error),
}

fn main() -> Result<(), Error> {
    let config_path = args().nth(1).expect("A config file must be supplied on the command line");
    let config: Config = toml::from_str(&fs::read_to_string(config_path)?)?;

    thread::scope(|s| {
        for camera in config.cameras {
            s.spawn(move |_| {
                // TODO handle these errors
                camera_main(&camera)
            });
        }
    }).unwrap();

    Ok(())
}

fn camera_main(camera_config: &CameraConfig) -> Result<(), Error> {
    let mut camera = BcCamera::new_with_addr(camera_config.camera_addr)?;

    println!("{}: Connecting to camera at {}", camera_config.name, camera_config.camera_addr);

    camera.connect()?;
    camera.login(&camera_config.username, camera_config.password.as_deref())?;

    let bind_addr = &camera_config.bind_addr;
    println!("{}: Logged in to camera; awaiting connection on {}", camera_config.name, bind_addr);

    let listener = TcpListener::bind(bind_addr)?;
    let (mut out_socket, remote_addr) = listener.accept()?;

    println!("{}: Connected to {}, starting video stream", camera_config.name, remote_addr);

    camera.start_video(&mut out_socket)?;

    Ok(())
}
