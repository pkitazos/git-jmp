use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => u64::MAX,
    }
}
