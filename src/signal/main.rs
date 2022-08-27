#![deny(warnings)]

mod mapper;
use std::net::SocketAddr;

use std::sync::Arc;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::Method;
use tokio::net::TcpListener;

// async fn base_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//     match (req.method(), req.uri().path()) {
//         (&Method::GET, "/") => Ok(Response::new(Body::from(
//             "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
//         ))),
//         (&Method::POST, "/echo") => Ok(Response::new(req.into_body())),
//         (&Method::POST, "/echo/reversed") => {
//             let whole_body = hyper::body::to_bytes(req.into_body()).await?;
//
//             let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
//             Ok(Response::new(Body::from(reversed_body)))
//         }
//         _ => {
//             let mut not_found = Response::default();
//             *not_found.status_mut() = StatusCode::NOT_FOUND;
//             Ok(not_found)
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let listener = TcpListener::bind(addr).await?;

    let m = Arc::new(mapper::Mapper::new());
    loop {
        let (stream, _) = listener.accept().await?;

        let m_clone = Arc::clone(&m);
        tokio::task::spawn(async move {
            if let Err(err) = Http::new().serve_connection(stream, service_fn( move |req| {
                (*m_clone).handler(req)
            })).await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}