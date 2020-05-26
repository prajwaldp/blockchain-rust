// pub mod time {
//     use std::time::{SystemTime, UNIX_EPOCH};

//     pub fn now() -> u128 {
//         let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
//         duration.as_secs() as u128 * 1000 + duration.subsec_millis() as u128
//     }
// }

pub mod traits;
