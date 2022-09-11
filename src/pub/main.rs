mod platform;
mod backend;
mod audio_sender;

use actix_web::body::MessageBody;
use platform::audio_capture_client::AudioCaptureClient;
use std::error;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc;
use wasapi::*;
use opus::{Decoder, Encoder};

#[macro_use]
extern crate log;
extern crate core;

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

    let mut origin_outfile = File::create("../origin_recorded.raw")?;
    let mut decode_outfile = File::create("../decode_recorded.raw")?;
    info!("Saving captured raw data to 'recorded.raw'");

    let mut encoder = Encoder::new(48000, opus::Channels::Mono, opus::Application::Audio).unwrap();
    let mut decoder = Decoder::new(48000, opus::Channels::Mono).unwrap();

	// let mut encoder = opus::Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio).unwrap();
    let mut frames: Vec<i16> = Vec::with_capacity(960);
    loop {
        match rx_capt.recv() {
            Ok(chunk) => {
                debug!("writing to file");
                origin_outfile.write_all(&chunk)?;

                let mut i = 0;
                while i < chunk.len() {
                    let mut buffer = [chunk[i], chunk[i+1]];
                    frames.push(i16::from_ne_bytes(buffer));
                    if frames.len() == 960 {
                        info!("frames data len is {:?}\n {:?}", &frames.len(), &frames);
                        let mut encode_data: Vec<u8> = vec![0; 480];
                        let encode_len = encoder.encode(&frames[..], encode_data.as_mut_slice())?;
                        encode_data.truncate(encode_len);
                        info!("encoded data len is {:?}\n {:?}", encode_len, &encode_data);

                        let mut decode_data: Vec<i16> = vec![0; 1920];
                        let decode_len = decoder.decode(&encode_data[..], decode_data.as_mut_slice(), false)?;
                        info!("decoded data len is {:?}\n {:?}", decode_len, &decode_data);

                        // outfile.write_all(&encode_data)?;
                        for i in 0..decode_len {
                            decode_outfile.write_all(&i16::to_ne_bytes(decode_data[i]))?;
                        }
                        frames.clear();
                    }
                    i += 2;
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