use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ReqOffer {
    pub sdp: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReqGetOffer{
   pub id: u32,
}


pub struct Offer {
    pub sdp: String,
    pub add_time: u64,
}

// pub struct Answer {
//     pub id: i32,
//     pub sdp: String
// }
