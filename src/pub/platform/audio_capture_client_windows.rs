use std::collections::VecDeque;
use std::error;
use wasapi::*;
use std::thread;
use super::audio_capture_client::AudioCaptureClient;


type Res<T> = Result<T, Box<dyn error::Error>>;
pub struct AudioCaptureClientWindows {
    chunksize: usize
}

fn start_async(tx_capt: std::sync::mpsc::SyncSender<Vec<u8>>, chunksize: usize) {
    // Capture
    let _handle = thread::Builder::new()
    .name("Capture".to_string())
    .spawn(move || {
        let result = capture_loop(tx_capt, chunksize);
        if let Err(err) = result {
            error!("Capture failed with error {}", err);
        }
    });
}

// Capture loop, capture samples and send in chunks of "chunksize" frames to channel
fn capture_loop(tx_capt: std::sync::mpsc::SyncSender<Vec<u8>>, chunksize: usize) -> Res<()> {
    // Use `Direction::Capture` for normal capture,
    // or `Direction::Render` for loopback mode (for capturing from a playback device).
    let device = get_default_device(&Direction::Render)?;

    let mut audio_client = device.get_iaudioclient()?;

    let desired_format = WaveFormat::new(16, 16, &SampleType::Int, 48000, 1);

    let blockalign = desired_format.get_blockalign();
    debug!("Desired capture format: {:?}", desired_format);

    let (def_time, min_time) = audio_client.get_periods()?;
    debug!("default period {}, min period {}", def_time, min_time);

    audio_client.initialize_client(
        &desired_format,
        min_time as i64,
        &Direction::Capture,
        &ShareMode::Shared,
        true,
    )?;
    debug!("initialized capture");

    let h_event = audio_client.set_get_eventhandle()?;

    let buffer_frame_count = audio_client.get_bufferframecount()?;

    let render_client = audio_client.get_audiocaptureclient()?;
    let mut sample_queue: VecDeque<u8> = VecDeque::with_capacity(
        100 * blockalign as usize * (1024 + 2 * buffer_frame_count as usize),
    );
    let session_control = audio_client.get_audiosessioncontrol()?;

    debug!("state before start: {:?}", session_control.get_state());
    audio_client.start_stream()?;
    debug!("state after start: {:?}", session_control.get_state());

    loop {
        while sample_queue.len() > (blockalign as usize * chunksize as usize) {
            debug!("pushing samples");
            let mut chunk = vec![0u8; blockalign as usize * chunksize as usize];
            for element in chunk.iter_mut() {
                *element = sample_queue.pop_front().unwrap();
            }
            tx_capt.send(chunk)?;
        }
        trace!("capturing");
        render_client.read_from_device_to_deque(blockalign as usize, &mut sample_queue)?;
        if h_event.wait_for_event(3000).is_err() {
            error!("timeout error, stopping capture");
            audio_client.stop_stream()?;
            break;
        }
    }
    Ok(())
}

impl AudioCaptureClientWindows {
    pub fn new() -> AudioCaptureClientWindows {
        let chunksize: usize = 4096;
        AudioCaptureClientWindows{chunksize}
    }
}

impl AudioCaptureClient for AudioCaptureClientWindows {
    fn start(&self, tx_capt: std::sync::mpsc::SyncSender<Vec<u8>>) {
        start_async(tx_capt, self.chunksize);
    }
}