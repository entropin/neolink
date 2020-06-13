use err_derive::Error;
use log::*;
use neolink::bc_protocol::BcCamera;
use neolink::gst::RtspServer;
use neolink::Never;
use std::fs;
use std::io::Write;
use std::time::Duration;
use structopt::StructOpt;

mod cmdline;
mod config;

use cmdline::Opt;
use config::{CameraConfig, Config};

***REMOVED***[derive(Debug, Error)]
pub enum Error {
    ***REMOVED***[error(display = "Configuration parsing error")]
    ConfigError(***REMOVED***[error(source)] toml::de::Error),
    ***REMOVED***[error(display = "Communication error")]
    ProtocolError(***REMOVED***[error(source)] neolink::Error),
    ***REMOVED***[error(display = "I/O error")]
    IoError(***REMOVED***[error(source)] std::io::Error),
}

fn main() -> Result<(), Error> {
    env_logger::init();

    info!(
        "Neolink {} {}",
        env!("NEOLINK_VERSION"),
        env!("NEOLINK_PROFILE")
    );

    let opt = Opt::from_args();
    let config: Config = toml::from_str(&fs::read_to_string(opt.config)?)?;

    let rtsp = &RtspServer::new();

    crossbeam::scope(|s| {
        for camera in config.cameras {
            s.spawn(move |_| {
                // TODO handle these errors
                let mut output = rtsp.add_stream(&camera.name).unwrap(); // TODO
                camera_loop(&camera, &mut output)
            });
        }

        rtsp.run(&config.bind_addr);
    })
    .unwrap();

    Ok(())
}

fn camera_loop(camera_config: &CameraConfig, output: &mut dyn Write) -> Result<Never, Error> {
    let min_backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(15);
    let mut current_backoff = min_backoff;

    loop {
        let cam_err = camera_main(camera_config, output).unwrap_err();
        // Authentication failures are permanent; we retry everything else
        if cam_err.connected {
            current_backoff = min_backoff;
        }
        match cam_err.err {
            neolink::Error::AuthFailed => {
                error!(
                    "Authentication failed to camera {}, not retrying",
                    camera_config.name
                );
                return Err(cam_err.err.into());
            }
            _ => error!(
                "Error streaming from camera {}, will retry in {}s: {}",
                camera_config.name,
                current_backoff.as_secs(),
                cam_err.err
            ),
        }

        std::thread::sleep(current_backoff);
        current_backoff = std::cmp::min(max_backoff, current_backoff * 2);
    }
}

struct CameraErr {
    connected: bool,
    err: neolink::Error,
}

fn camera_main(camera_config: &CameraConfig, output: &mut dyn Write) -> Result<Never, CameraErr> {
    let mut connected = false;
    (|| {
        let mut camera = BcCamera::new_with_addr(camera_config.camera_addr)?;
        if let Some(timeout) = camera_config.timeout {
            camera.set_rx_timeout(timeout);
        }

        println!(
            "{}: Connecting to camera at {}",
            camera_config.name, camera_config.camera_addr
        );
        camera.connect()?;

        camera.login(&camera_config.username, camera_config.password.as_deref())?;

        connected = true;

        println!(
            "{}: Connected to camera, starting video stream",
            camera_config.name
        );
        camera.start_video(output)
    })()
    .map_err(|err| CameraErr { connected, err })
}
