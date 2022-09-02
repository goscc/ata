use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use hyper::{Body, Request, Response};
use serde_json::json;
use crate::mapper::bean::{Answer, Offer, ReqAnswer, ReqGetAnswer, ReqGetOffer, ReqOffer};
use crate::Method;


type R = Result<Response<Body>, hyper::Error>;

pub struct Mapper {
    pub id_to_offer: Mutex<HashMap<u32, Offer>>,
    pub id_to_answer: Mutex<HashMap<u32, Answer>>,
    pub counter: Mutex<u32>
}

impl Mapper{
    pub fn new() -> Mapper {
        Mapper{
            id_to_offer: Mutex::new(HashMap::new()),
            id_to_answer: Mutex::new(HashMap::new()),
            counter:Mutex::new(0),
        }
    }

    pub async fn handler(&self, req :Request<Body>) -> R  {
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/pub/offer") => pub_offer(self, req).await,
            (&Method::POST, "/sub/get/offer") => sub_get_offer(self, req).await,
            (&Method::POST, "/sub/answer") => sub_answer(self, req).await,
            (&Method::POST, "/sub/get/answer") => pub_get_answer(self, req).await,
            _ => Ok(Response::new(Body::from("not found"))),
        }
    }

    pub fn create_id(&self) -> u32{
        let mut m = self.counter.lock().unwrap();
        *m = *m + 1;
        *m
    }
}


fn error_resp(stat :u32, msg: &str) -> R {
    let resp_json = json!({
       "stat": stat,
       "msg": msg,
   });

    Ok(Response::new(Body::from(resp_json.to_string())))
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

//sub/answer
async fn sub_answer(m: &Mapper, req :Request<Body>) -> R {

    let req_body = hyper::body::to_bytes(req.into_body()).await?;
    let r  = serde_json::from_slice( &req_body);
    if r.is_err() {
        return error_resp(1, "request body is not answer")
    }

    let req_answer: ReqAnswer = r.unwrap();

    {
        let ito = m.id_to_offer.lock().unwrap();
        if ito.contains_key(&req_answer.id) {
            return error_resp(1, "id is duplicate, id maps to offer")
        }
    }

    {
        //这块最好的做法应该是server保存offer已经被pub接收的状态
        //发送pub的answer上一步必须是要这个状态，否则会被判定无效
        let mut ita = m.id_to_answer.lock().unwrap();

        let r  = SystemTime::now().duration_since(UNIX_EPOCH);
        if r.is_err() {
            return error_resp(2, "server error, get time fail");
        }

        let cur_time: Duration = r.unwrap();

        let r = ita.insert(req_answer.id, Answer{
            sdp: req_answer.sdp,
            add_time: cur_time.as_secs(),
        });

        //todo: 这个时候是不是没有办法通知pub端，pub应该是需要接收这个状态
        if !r.is_none(){
            return error_resp(2, "server error, add answer fail");
        }
    }

    let resp_json = json!({
        "stat": 0,
        "msg": "success",
        "data": {
            "id": req_answer.id,
        }
    });
    Ok(Response::new(Body::from(resp_json.to_string())))
}


//pub/get/answer
async fn pub_get_answer(m: &Mapper, req :Request<Body>) -> R {

    let req_body = hyper::body::to_bytes(req.into_body()).await?;
    let r  = serde_json::from_slice( &req_body);
    if r.is_err() {
        return error_resp(1, "request body is not answer")
    }

    let rga : ReqGetAnswer = r.unwrap();
    let mut ita = m.id_to_answer.lock().unwrap();
    let o = ita.remove(&rga.id);
    if o.is_none() {
        return error_resp(2, "do not exit this answer id")
    }

    let answer = o.unwrap();

    let resp_json = json!({
        "stat": 0,
        "msg": "success",
        "data": {
            "sdp": answer.sdp,
        }
    });
    Ok(Response::new(Body::from(resp_json.to_string())))
}