***REMOVED***[macro_use] extern crate validator_derive;
***REMOVED***[macro_use] extern crate lazy_static;

use env_logger::Env;
use err_derive::Error;
use log::*;
use neolink::bc_protocol::BcCamera;
use neolink::gst::{MaybeAppSrc, RtspServer, StreamFormat};
use neolink::Never;
use std::fs;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use validator::Validate;
use gio::TlsAuthenticationMode;

mod cmdline;
mod config;

use cmdline::Opt;
use config::{UserConfig, CameraConfig, Config};

***REMOVED***[derive(Debug, Error)]
pub enum Error {
    ***REMOVED***[error(display = "Configuration parsing error")]
    ConfigError(***REMOVED***[error(source)] toml::de::Error),
    ***REMOVED***[error(display = "Communication error")]
    ProtocolError(***REMOVED***[error(source)] neolink::Error),
    ***REMOVED***[error(display = "I/O error")]
    IoError(***REMOVED***[error(source)] std::io::Error),
    ***REMOVED***[error(display = "Validation error")]
    ValidationError(***REMOVED***[error(source)] validator::ValidationErrors),
}

fn main() -> Result<(), Error> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    info!(
        "Neolink {} {}",
        env!("NEOLINK_VERSION"),
        env!("NEOLINK_PROFILE")
    );

    let opt = Opt::from_args();
    let config: Config = toml::from_str(&fs::read_to_string(opt.config)?)?;

    match config.validate() {
        Ok(_) => (),
        Err(e) => return Err(Error::ValidationError(e)),
    };

    let rtsp = &RtspServer::new();

    set_up_tls(&config, &rtsp);

    set_up_users(&config.users, &rtsp);

    crossbeam::scope(|s| {
        for camera in config.cameras {
            let stream_format = match &*camera.format {
                "h264"|"H264" => StreamFormat::H264,
                "h265"|"H265" => StreamFormat::H265,
                custom_format @ _ => StreamFormat::Custom(custom_format.to_string())
            };
            let permitted_user = get_permitted_users(&config.users, &camera.permitted_users);

            // Let subthreads share the camera object; in principle I think they could share
            // the object as it sits in the config.cameras block, but I have not figured out the
            // syntax for that.
            let arc_cam = Arc::new(camera);

            // Set up each main and substream according to all the RTSP mount paths we support
            if arc_cam.stream == "both" || arc_cam.stream == "mainStream" {
                let paths = &[
                    &arc_cam.name,
                    &*format!("{}/mainStream", arc_cam.name),
                ];
                let mut output = rtsp.add_stream(paths, &stream_format, &permitted_user).unwrap();
                let main_camera = arc_cam.clone();
                s.spawn(move |_| camera_loop(&*main_camera, &mut output));
            }
            if arc_cam.stream == "both" || arc_cam.stream == "subStream" {
                let paths = &[&*format!("{}/subStream", arc_cam.name)];
                let mut output = rtsp.add_stream(paths, &stream_format, &permitted_user).unwrap();
                let sub_camera = arc_cam.clone();
                s.spawn(move |_| camera_loop(&*sub_camera, &mut output));
            }
        }

        rtsp.run(&config.bind_addr, config.bind_port);
    })
    .unwrap();

    Ok(())
}

fn camera_loop(camera_config: &CameraConfig, output: &mut MaybeAppSrc) -> Result<Never, Error> {
    let min_backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(15);
    let mut current_backoff = min_backoff;

    loop {
        let cam_err = camera_main(camera_config, output).unwrap_err();
        output.on_stream_error();
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

fn set_up_tls(config: &Config, rtsp: &RtspServer) {
    let tls_client_auth = match &config.tls_client_auth as &str {
        "request" => TlsAuthenticationMode::Requested,
        "require" => TlsAuthenticationMode::Required,
        "none"|_ => TlsAuthenticationMode::None,
    };
    rtsp.set_tls(&config.certificate, tls_client_auth).expect("Failed to set up TLS");
}

fn set_up_users(users: &Vec<UserConfig>, rtsp: &RtspServer) {
    // Setting up users
    let mut credentials = vec![];
    for user in users {
        credentials.push((&user.name, &user.pass));
    }
    rtsp.set_credentials(&credentials).expect("Failed to set up users.");
}

fn get_permitted_users(users: &Vec<UserConfig>, current_permitted_users: &Vec<String>) -> Vec<String> {
    // This is required to handle the special case of "anyone"
    // Special set up of "anyone"
    // If in the camera config there is the user anyone
    // Then we add all users to the cameras config. including unauth
    let mut new_permitted_users = vec![];
    if current_permitted_users.contains(&"anyone".to_string()) {
        for credentials in users {
            let user: &str = &credentials.name;
            new_permitted_users.push(user.to_string());
        }
        new_permitted_users.push("unauth".to_string());
    } else {
        new_permitted_users.append(&mut current_permitted_users.clone());
    }
    new_permitted_users.sort();
    new_permitted_users.dedup();

    new_permitted_users
}

fn camera_main(camera_config: &CameraConfig, output: &mut dyn Write) -> Result<Never, CameraErr> {
    let mut connected = false;
    (|| {
        let mut camera = BcCamera::new_with_addr(camera_config.camera_addr)?;
        if let Some(timeout) = camera_config.timeout {
            camera.set_rx_timeout(timeout);
        }

        info!(
            "{}: Connecting to camera at {}",
            camera_config.name, camera_config.camera_addr
        );
        camera.connect()?;

        camera.login(&camera_config.username, camera_config.password.as_deref())?;

        connected = true;

        info!(
            "{}: Connected to camera, starting video stream {}",
            camera_config.name, camera_config.stream
        );
        camera.start_video(output, &camera_config.stream)
    })()
    .map_err(|err| CameraErr { connected, err })
}
