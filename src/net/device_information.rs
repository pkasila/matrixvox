extern crate serde;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceInformation {
    pub product_id: String,
    pub serial_number: String,
    pub vox_size: [i32; 3],
}