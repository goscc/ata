mod platform;
mod backend;
use actix_web::body::MessageBody;
use platform::audio_capture_client::AudioCaptureClient;
use std::error;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc;
use wasapi::*;
use opus::Encoder;

#[macro_use]
extern crate log;
use simplelog::*;

use crate::platform::audio_capture_client::get_client;

type Res<T> = Result<T, Box<dyn error::Error>>;

//Main loop
fn main() -> Res<()> {
    let _ = SimpleLogger::init(
        LevelFilter::Trace,
        ConfigBuilder::new()
            .set_time_format_str("%H:%M:%S%.3f")
            .build(),
    );

    initialize_mta()?;

    let (tx_capt, rx_capt): (
        std::sync::mpsc::SyncSender<Vec<u8>>,
        std::sync::mpsc::Receiver<Vec<u8>>,
    ) = mpsc::sync_channel(2);

    let client = get_client();
    client.start(tx_capt);

    let mut outfile = File::create("../../recorded.raw")?;
    info!("Saving captured raw data to 'recorded.raw'");

    let mut encoder = Encoder::new(48000, opus::Channels::Stereo, opus::Application::LowDelay).unwrap();
	// let mut encoder = opus::Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio).unwrap();
    loop {
        match rx_capt.recv() {
            Ok(chunk) => {
                debug!("writing to file");
                let mut index = 0;
                let mut buffer = [0u8; 2];
                let mut frames: Vec<i16> = Vec::with_capacity(chunk.len()/2);
                for i in 0..chunk.len() {
                    buffer[index] = chunk[i];
                    if index == 1 {
                        index = 0;
                        frames.push(i16::from_le_bytes(buffer));
                        if frames.len() == 480 {
                            info!("reciving input {} size.", frames.len());
                            let output = encoder.encode_vec(&frames[..], frames.len())?;
                            info!("reciving {} size.", output.len());
                            outfile.write_all(&output)?;
                            frames.clear();
                        }
                    } else {
                        index += 1;
                    }
                }
            }
            Err(err) => {
                error!("Some error {}", err);
                return Ok(());
            }
        }
    }
}

// use actix_web::{guard, web, App, HttpServer, Responder, HttpResponse, get};
// use actix_files::{NamedFile};
// use backend::server;

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| {
//         App::new().service(server::index)
//     })
//     .bind(("127.0.0.1", 8080))?
//     .run()
//     .await
// }