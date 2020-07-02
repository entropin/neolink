//! This module provides an "RtspServer" abstraction that allows consumers of its API to feed it
//! data using an ordinary std::io::Write interface.
pub use self::maybe_app_src::MaybeAppSrc;

use gstreamer::prelude::Cast;
use gstreamer::Bin;
use gstreamer_app::AppSrc;
//use gstreamer_rtsp::RTSPLowerTrans;
use gstreamer_rtsp_server::prelude::*;
use gstreamer_rtsp_server::{RTSPAuth, RTSPMediaFactory, RTSPServer as GstRTSPServer};
use log::debug;
use std::io;
use std::io::Write;

type Result<T> = std::result::Result<T, ()>;

pub struct RtspServer {
    server: GstRTSPServer,
}

pub enum StreamFormat {
    H264,
    H265,
    Custom(String)
}

impl RtspServer {
    pub fn new() -> RtspServer {
        gstreamer::init().expect("Gstreamer should not explode");
        RtspServer {
            server: GstRTSPServer::new(),
        }
    }

    pub fn add_stream(&self, names: &[&str], stream_format: &StreamFormat) -> Result<MaybeAppSrc> {
        let mounts = self
            .server
            .get_mount_points()
            .expect("The server should have mountpoints");

        let launch_str = match stream_format {
            StreamFormat::H264 => "! h264parse ! rtph264pay name=pay0",
            StreamFormat::H265 => "! h265parse ! rtph265pay name=pay0",
            StreamFormat::Custom(custom_format) => custom_format,
        };

        let factory = RTSPMediaFactory::new();
        //factory.set_protocols(RTSPLowerTrans::TCP);
        factory.set_launch(&format!("{}{}{}{}",
            "( ",
            "appsrc name=writesrc is-live=true block=true emit-signals=false max-bytes=0 do-timestamp=true ",
            launch_str,
            " )"
        ));
        factory.set_shared(true);

        // TODO maybe set video format via
        // https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/master/examples/src/bin/appsrc.rs***REMOVED***L66

        // Create a MaybeAppSrc: Write which we will give the caller.  When the backing AppSrc is
        // created by the factory, fish it out and give it to the waiting MaybeAppSrc via the
        // channel it provided.  This callback may be called more than once by Gstreamer if it is
        // unhappy with the pipeline, so keep updating the MaybeAppSrc.
        let (maybe_app_src, tx) = MaybeAppSrc::new_with_tx();
        factory.connect_media_configure(move |_factory, media| {
            debug!("RTSP: media was configured");
            let bin = media
                .get_element()
                .expect("Media should have an element")
                .dynamic_cast::<Bin>()
                .expect("Media source's element should be a bin");
            let app_src = bin
                .get_by_name_recurse_up("writesrc")
                .expect("write_src must be present in created bin")
                .dynamic_cast::<AppSrc>()
                .expect("Source element is expected to be an appsrc!");
            let _ = tx.send(app_src); // Receiver may be dropped, don't panic if so
        });

        for name in names {
            mounts.add_factory(&format!("/{}", name), &factory);
        }

        Ok(maybe_app_src)
    }

    pub fn set_credentials(&mut self, user_pass: Option<(&str, &str)>) -> Result<()> {
        let auth = user_pass.map(|(user, pass)| {
            let auth = RTSPAuth::new();
            /*
            let perm = RTSPToken::new(
                ...
            );
            auth.add_basic(RTSPAuth::make_basic(user, pass).as_str(), &perm);
            */
            // TODO TLS https://thiblahute.github.io/GStreamer-doc/gst-rtsp-server-1.0/rtsp-server.html?gi-language=c
            auth
        });

        self.server.set_auth(auth.as_ref());

        Ok(())
    }

    pub fn run(&self, bind_addr: &str, bind_port: u16) {
        self.server.set_address(bind_addr);
        self.server.set_service(&format!("{}", bind_port));
        // Attach server to default Glib context
        self.server.attach(None);

        // Run the Glib main loop.
        let main_loop = glib::MainLoop::new(None, false);
        main_loop.run();
    }
}

mod maybe_app_src {
    use super::*;
    use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

    /// A Write implementation around AppSrc that also allows delaying the creation of the AppSrc
    /// until later, discarding written data until the AppSrc is provided.
    pub struct MaybeAppSrc {
        rx: Receiver<AppSrc>,
        app_src: Option<AppSrc>,
    }

    impl MaybeAppSrc {
        /// Creates a MaybeAppSrc.  Also returns a Sender that you must use to provide an AppSrc as
        /// soon as one is available.  When it is received, the MaybeAppSrc will start pushing data
        /// into the AppSrc when write() is called.
        pub fn new_with_tx() -> (Self, SyncSender<AppSrc>) {
            let (tx, rx) = sync_channel(3); // The sender should not send very often
            (MaybeAppSrc { rx, app_src: None }, tx)
        }

        /// Flushes data to Gstreamer on a problem communicating with the underlying video source.
        pub fn on_stream_error(&mut self) {
            if let Some(src) = self.try_get_src() {
                // Ignore "errors" from Gstreamer such as FLUSHING, which are not really errors.
                let _ = src.end_of_stream();
            }
        }

        /// Attempts to retrieve the AppSrc that should be passed in by the caller of new_with_tx
        /// at some point after this struct has been created.  At that point, we swap over to
        /// owning the AppSrc directly.  This function handles either case and returns the AppSrc,
        /// or None if the caller has not yet sent one.
        fn try_get_src(&mut self) -> Option<&AppSrc> {
            while let Some(src) = self.rx.try_recv().ok() {
                self.app_src = Some(src);
            }
            self.app_src.as_ref()
        }
    }

    impl Write for MaybeAppSrc {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            // If we have no AppSrc yet, throw away the data and claim that it was written
            let app_src = match self.try_get_src() {
                Some(src) => src,
                None => return Ok(buf.len()),
            };
            let mut gst_buf = gstreamer::Buffer::with_size(buf.len()).unwrap();
            {
                let gst_buf_mut = gst_buf.get_mut().unwrap();
                let mut gst_buf_data = gst_buf_mut.map_writable().unwrap();
                gst_buf_data.copy_from_slice(buf);
            }
            let res = app_src.push_buffer(gst_buf); //.map_err(|e| io::Error::new(io::ErrorKind::Other, Box::new(e)))?;
            if res.is_err() {
                self.app_src = None;
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
