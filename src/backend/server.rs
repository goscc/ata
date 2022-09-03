use actix_web::{guard, web, App, HttpServer, Responder, HttpResponse, get};
use actix_files::{NamedFile};

#[get("/")]
async fn index() -> std::io::Result<NamedFile> {
    Ok(NamedFile::open(std::path::PathBuf::from("./pages/client.html"))?)
}