extern crate serde;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Pack {
    anim_rate: usize,
    slices: usize,
    data: Vec<Vec<Vec<[u8; 8]>>>,
}
