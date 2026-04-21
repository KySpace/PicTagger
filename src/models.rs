use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ImageRecord {
    pub id: Uuid,
    pub image_data: String,
    pub ib: f64,
    pub source: String,
    pub index: i32,
    pub frequency: f64,
    pub weight: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ImageRecord {
    pub fn new(image_data: String, source: String) -> Self {
        let ts = now_millis();
        Self {
            id: Uuid::new_v4(),
            image_data,
            ib: 0.0,
            source,
            index: 0,
            frequency: 0.0,
            weight: 0.0,
            created_at: ts,
            updated_at: ts,
        }
    }
}

pub fn now_millis() -> i64 {
    js_sys::Date::now() as i64
}
