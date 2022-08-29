use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Offer {
    pub sdp: String,
}

// pub struct Answer {
//     pub id: i32,
//     pub sdp: String
// }
