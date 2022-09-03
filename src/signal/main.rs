#![deny(warnings)]

extern crate core;

mod mapper;

use std::net::SocketAddr;

use std::sync::{Arc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response};
use tokio::net::TcpListener;
use tokio::time;
use tokio::time::{Instant};
use crate::mapper::mapper::Mapper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let listener = TcpListener::bind(addr).await?;

    let m_arc = Arc::new(Mapper::new());

    let m_clone = Arc::clone(&m_arc);
    tokio::task::spawn(
        async move {
            let mut interval = time::interval_at(Instant::now() + Duration::from_secs(1), Duration::from_secs(1));
            loop {

                interval.tick().await;

                let tr = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                {
                    let mut ita = m_clone.id_to_answer.lock().unwrap();
                    ita.retain(|_, v| tr.as_secs() - v.add_time < 30);
                }
                {
                    let mut ito = m_clone.id_to_offer.lock().unwrap();
                    ito.retain(|_, v| tr.as_secs() - v.add_time < 30);
                }
            }
        }
    );

    loop {
        let (stream, _) = listener.accept().await?;
        let m_clone = Arc::clone(&m_arc);
        tokio::task::spawn(async move {
            if let Err(err) = Http::new().serve_connection(stream, service_fn( move |req| {
                let m_clone2 = Arc::clone(&m_clone);
                mapper_handler(m_clone2, req)
            })).await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn mapper_handler(m:Arc<Mapper>, req: Request<Body>) -> Result<Response<Body>, hyper::Error>  {
    m.handler(req).await
}