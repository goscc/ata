pub mod mapper;


use hyper::{Body, Request, Response};
use crate::Method;



pub struct Mapper {
    // get : Mutex<HashMap<str, F>>,
    // post: Mutex<HashMap<str, F>>,
}

impl Mapper{
    pub fn new() -> Mapper {
        Mapper{}
    }

    pub async fn handler(self, req :Request<Body>) -> Result<Response<Body>, hyper::Error>  {
        match req.method() {
            &Method::GET => Ok(Response::new(Body::from("GET"))),
            &Method::POST => Ok(Response::new(Body::from("POST"))),
            _=>Ok(Response::new(Body::from("not found"))),
        }
    }
}