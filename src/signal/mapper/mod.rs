pub mod mapper;
mod bean;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use hyper::{Body, Request, Response};
use serde_json::json;
use crate::mapper::bean::{Offer, ReqGetOffer, ReqOffer};
use crate::Method;


type R = Result<Response<Body>, hyper::Error>;

pub struct Mapper {
    pub id_to_offer: Mutex<HashMap<u32, Offer>>,
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
            (&Method::POST, "/sub/get/offer") => sub_get_offer(self, req).await,
            _ => Ok(Response::new(Body::from("not found"))),
        }
    }

    pub fn create_id(&self) -> u32{
        let mut m = self.counter.lock().unwrap();
        *m = *m + 1;
        *m
    }
}


//sub/get/offer
async fn sub_get_offer(m: &Mapper, req :Request<Body>) -> R {
    let req_body = hyper::body::to_bytes(req.into_body()).await?;
    let r = serde_json::from_slice(&req_body);

    if r.is_err() {
        return error_resp(1, "request body is not offer")
    }

    let rgo :ReqGetOffer = r.unwrap();

    let mut ito = m.id_to_offer.lock().unwrap();
    let op = ito.remove(&rgo.id);

    if op.is_none() {
        //todo remove
        return error_resp(1, "do not exit this offer id")
    }

    let of : Offer = op.unwrap();


    let resp_json = json!({
        "stat": 0,
        "msg": "success",
        "data": {
            "sdp": of.sdp,
        }
    });
    //todo: remove

    Ok(Response::new(Body::from(resp_json.to_string())))
}

//pub/offer
async fn pub_offer(m: &Mapper, req :Request<Body>) -> R {
    let req_body = hyper::body::to_bytes(req.into_body()).await?;

    let r  = serde_json::from_slice( &req_body);
    if r.is_err() {
        return error_resp(1, "request body is not offer")
    }

    let ro :ReqOffer = r.unwrap();


    let id = m.create_id();

    let mut ito = m.id_to_offer.lock().unwrap();

    let r  = SystemTime::now().duration_since(UNIX_EPOCH);
    if r.is_err() {
        return error_resp(2, "server error, get time fail");
    }

    let cur_time: Duration = r.unwrap();

    let offer = Offer{
        sdp: ro.sdp,
        add_time: cur_time.as_secs(),
    };

    ito.insert(id, offer);

    let resp_json = json!({
        "stat": 0,
        "msg": "success",
        "data": {
            "id": id,
        }
    });
    Ok(Response::new(Body::from(resp_json.to_string())))
}


fn error_resp(stat :u32, msg: &str) -> R {
   let resp_json = json!({
       "stat": stat,
       "msg": msg,
   });

   Ok(Response::new(Body::from(resp_json.to_string())))
}