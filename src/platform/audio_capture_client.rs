use super::audio_capture_client_windows::AudioCaptureClientWindows;

pub trait AudioCaptureClient {
    fn start(&self, tx_capt: std::sync::mpsc::SyncSender<Vec<u8>>);
}

pub fn get_client() -> impl AudioCaptureClient {
    // if window
    let client = AudioCaptureClientWindows::new();
    client
}