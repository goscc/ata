pub mod mapper;


use std::collections::HashMap;
use std::future::Future;
use hyper::{Body, Request, Response};
use crate::Method;


type F = dyn FnMut(Request<Body>) -> dyn Future;

pub struct Mapper {
    get : HashMap<str, F>,
    post: HashMap<str, F>,
}

impl Mapper{
    pub fn handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error>{
        match req.method() {
            // &Method::GET => ,
            // &Method::POST => ,
            _ => Ok(Response::new(Body::from("lqq")))
        }
    }

    // fn register(m: Method, f : F) {
    //
    //
    // }
}