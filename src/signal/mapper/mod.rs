pub mod mapper;
mod bean;

use std::collections::HashMap;
use std::sync::Mutex;
use hyper::{Body, Request, Response};
use serde_json::json;
use crate::mapper::bean::Offer;
use crate::Method;


type R = Result<Response<Body>, hyper::Error>;

pub struct Mapper {
    pub id_to_offer: Mutex<HashMap<u32, String>>,
    pub counter: Mutex<u32>
}

impl Mapper{
    pub fn new() -> Mapper {
        Mapper{
           id_to_offer: Mutex::new(HashMap::new()),
            counter:Mutex::new(0),
        }
    }

    pub async fn handler(&self, req :Request<Body>) -> R  {
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/pub/offer") => pub_offer(self, req).await,
            _ => Ok(Response::new(Body::from("not found"))),
        }
    }

    pub fn create_id(&self) -> u32{
        let mut m = self.counter.lock().unwrap();
        *m = *m + 1;
        *m
    }
}

//pub/offer
async fn pub_offer(m: &Mapper, req :Request<Body>) -> R {
    let req_body = hyper::body::to_bytes(req.into_body()).await?;

    let Err(e) = serde_json::from_slice( &req_body);
    if Err(e) {
        return error(1, "sdp is nil")
    }
    let s :Offer = Ok(s).unwrap();

    let id = m.create_id();

    let mut ito = m.id_to_offer.lock().unwrap();
    ito.insert(id, s.sdp);

    let resp_json = json!({
        "stat": 0,
        "msg": "success",
        "data": {
            "id": id,
        }
    });
    Ok(Response::new(Body::from(resp_json.to_string())))
}


fn error(stat :u32, msg: &str) -> R {
   let resp_json = json!({
       "stat": stat,
       "msg": msg,
   });

   Ok(Response::new(Body::from(resp_json.to_string())))
}


//p