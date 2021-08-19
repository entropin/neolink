pub use super::errors::Error;
use super::{BcCamera, BinarySubscriber, Result};
use crate::{
    bc::{model::*, xml::*},
    bcmedia::model::*,
    Never,
};

/// Convience type for the error raised by the [StreamOutput] trait
pub type StreamOutputError = Result<()>;

/// The method [`BcCamera::start_video()`] requires a structure with this trait to pass the
/// audio and video data back to
pub trait StreamOutput {
    /// This is the callback raised a complete media packet is received
    fn write(&mut self, media: BcMedia) -> StreamOutputError;
}

/// Convert the name name stream number
///
/// # Parameters
///
/// * `stream_name` - The name of the stream either `"mainStream"` for HD or `"subStream"` for SD
///
/// # Returns
///
/// u8 stream nummber
///
fn get_stream_num(stream_name: &str) -> u8 {
    let stream_num = match stream_name {
        "mainStream" => 0,
        "subStream" => 1,
        _ => 0,
    };

    stream_num
}

impl BcCamera {
    ///
    /// Starts the video stream
    ///
    /// # Parameters
    ///
    /// * `data_outs` - This should be a struct that implements the [`StreamOutput`] trait
    ///
    /// * `stream_name` - The name of the stream either `"mainStream"` for HD or `"subStream"` for SD
    ///
    /// # Returns
    ///
    /// This will block forever or return an error when the camera connection is dropped
    ///
    pub fn start_video<Outputs>(&self, data_outs: &mut Outputs, stream_name: &str) -> Result<Never>
    where
        Outputs: StreamOutput,
    {
        let connection = self
            .connection
            .as_ref()
            .expect("Must be connected to start video");
        let sub_video = connection.subscribe(MSG_ID_VIDEO)?;

        let stream_num = get_stream_num(stream_name);

        let start_video = Bc::new_from_xml(
            BcMeta {
                msg_id: MSG_ID_VIDEO,
                channel_id: self.channel_id,
                msg_num: self.new_message_num(),
                stream_type: stream_num,
                response_code: 0,
                class: 0x6414, // IDK why
            },
            BcXml {
                preview: Some(Preview {
                    version: xml_ver(),
                    channel_id: self.channel_id,
                    handle: 0,
                    stream_type: stream_name.to_string(),
                }),
                ..Default::default()
            },
        );

        sub_video.send(start_video)?;

        let mut media_sub = BinarySubscriber::from_bc_sub(&sub_video);

        loop {
            let bc_media = BcMedia::deserialize(&mut media_sub)?;
            // We now have a complete interesting packet. Send it to on the callback
            data_outs.write(bc_media)?;
        }
    }

    /// Capture a frame from the camera and return
    ///
    /// # Parameters
    ///
    /// * `stream_name` - The name of the stream either `"mainStream"` for HD or `"subStream"` for SD
    ///
    /// * `init_stream` - Ask camera to initiat a video stream for frame capture
    ///
    /// # Returns
    ///
    /// BcMediaIframe Package
    ///
    pub fn capture_frame(
        &self,
        stream_name: &str,
        init_stream: bool,
    ) -> Result<BcMediaIframe>
    {
        let connection = self
            .connection
            .as_ref()
            .expect("Must be connected to start video frame capturing");
        let sub_video = connection.subscribe(MSG_ID_VIDEO)?;

        let stream_num = get_stream_num(stream_name);

        let start_video = Bc::new_from_xml(
            BcMeta {
                msg_id: MSG_ID_VIDEO,
                channel_id: self.channel_id,
                msg_num: self.new_message_num(),
                stream_type: stream_num,
                response_code: 0,
                class: 0x6414, // IDK why
            },
            BcXml {
                preview: Some(Preview {
                    version: xml_ver(),
                    channel_id: self.channel_id,
                    handle: 0,
                    stream_type: stream_name.to_string(),
                }),
                ..Default::default()
            },
        );

        // Tell the camera to start sending video packages
        if init_stream {
            sub_video.send(start_video)?;
        }

        // Waits for media, the program will exit if timeoute
        let mut media_sub = BinarySubscriber::from_bc_sub(&sub_video);

        let max_retrys = 30;
        let mut retry_count = 0;

        // Give the stream some time to capture a full Iframe
        while retry_count >= max_retrys {
            let bc_media = BcMedia::deserialize(&mut media_sub)?;

            match bc_media {
                BcMedia::Iframe(payload) => {
                    return Ok(payload);
                }
                _ => {
                    // Retry
                    retry_count += 1;
                    println!("No full BcMediaIframe found - Retrying: {}", retry_count);
                }
            }
        }

        return Err(Error::Timeout);
    }
}
