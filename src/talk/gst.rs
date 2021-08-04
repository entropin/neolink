use gstreamer::{
    element_error, parse_launch, prelude::*, ClockTime, FlowError, FlowSuccess, MessageView,
    Pipeline, ResourceError, State,
};
use gstreamer_app::{AppSink, AppSinkCallbacks};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use super::errors::Error;
use byte_slice_cast::*;

pub(super) fn file_input(
    filename: &str,
    block_align: u16,
    sample_rate: u16,
) -> Result<Receiver<Vec<u8>>, Error> {
    let pipeline = create_pipeline(filename, block_align, sample_rate)?;
    let appsink = get_sink(&pipeline)?;
    let (tx, rx) = sync_channel(30);

    set_data_channel(&appsink, tx);

    std::thread::spawn(move || {
        let _ = start_pipeline(pipeline);
    });

    Ok(rx)
}

fn start_pipeline(pipeline: Pipeline) -> Result<(), Error> {
    pipeline.set_state(State::Playing)?;

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(ClockTime::NONE) {
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(State::Null)?;
                log::warn!(
                    "{:?}",
                    Error::Gstreamer {
                        error: err.error().to_string(),
                        debug: err.debug(),
                    }
                );
            }
            _ => (),
        }
    }

    pipeline.set_state(State::Null)?;

    Ok(())
}

fn get_sink(pipeline: &Pipeline) -> Result<AppSink, Error> {
    let sink = pipeline
        .by_name("thesink")
        .expect("There shoud be a `thesink`");
    sink.dynamic_cast::<AppSink>()
        .map_err(Error::GstreamerElement)
}

fn set_data_channel(appsink: &AppSink, tx: SyncSender<Vec<u8>>) {
    // Getting data out of the appsink is done by setting callbacks on it.
    // The appsink will then call those handlers, as soon as data is available.
    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            // Add a handler to the "new-sample" signal.
            .new_sample(move |appsink| {
                // Pull the sample in question out of the appsink's buffer.
                let sample = appsink.pull_sample().map_err(|_| FlowError::Eos)?;
                let buffer = sample.buffer().ok_or_else(|| {
                    element_error!(
                        appsink,
                        ResourceError::Failed,
                        ("Failed to get buffer from appsink")
                    );

                    FlowError::Error
                })?;

                // At this point, buffer is only a reference to an existing memory region somewhere.
                // When we want to access its content, we have to map it while requesting the required
                // mode of access (read, read/write).
                // This type of abstraction is necessary, because the buffer in question might not be
                // on the machine's main memory itself, but rather in the GPU's memory.
                // So mapping the buffer makes the underlying memory region accessible to us.
                // See: https://gstreamer.freedesktop.org/documentation/plugin-development/advanced/allocation.html
                let map = buffer.map_readable().map_err(|_| {
                    element_error!(
                        appsink,
                        ResourceError::Failed,
                        ("Failed to map buffer readable")
                    );

                    FlowError::Error
                })?;

                // We know what format the data in the memory region has, since we requested
                // it by setting the appsink's caps. So what we do here is interpret the
                // memory region we mapped as an array of signed 16 bit integers.
                let samples = map.as_slice_of::<u8>().map_err(|_| {
                    element_error!(
                        appsink,
                        ResourceError::Failed,
                        ("Failed to interprete buffer as S16 PCM")
                    );

                    FlowError::Error
                })?;

                // Ready!
                let _ = tx.send(samples.to_vec());

                Ok(FlowSuccess::Ok)
            })
            .build(),
    );
}

fn create_pipeline(filename: &str, block_align: u16, sample_rate: u16) -> Result<Pipeline, Error> {
    gstreamer::init()?;

    let launch_str = format!(
        "filesrc location={} \
        ! decodebin \
        ! audioconvert \
        ! audioresample \
        ! audio/x-raw,rate={},channels=1 \
        ! queue  \
        ! adpcmenc blockalign={} layout=dvi \
        ! appsink name=thesink",
        filename, sample_rate, block_align
    );

    // log::info!("{}", launch_str);

    // Parse the pipeline we want to probe from a static in-line string.
    // Here we give our audiotestsrc a name, so we can retrieve that element
    // from the resulting pipeline.
    let pipeline = parse_launch(&launch_str)?;
    let pipeline = pipeline
        .dynamic_cast::<Pipeline>()
        .map_err(Error::GstreamerElement)?;

    Ok(pipeline)
}