use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct FrequencyWeightPair {
    #[serde(default)]
    pub frequency: Option<f64>,
    #[serde(default)]
    pub weight: Option<f64>,
}

impl FrequencyWeightPair {
    pub fn blank() -> Self {
        Self {
            frequency: None,
            weight: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ImageRecord {
    pub id: Uuid,
    pub image_data: String,
    #[serde(default)]
    pub image_path: String,
    pub ib: f64,
    pub source: String,
    #[serde(default)]
    pub source_tag: String,
    #[serde(default = "default_image_tags")]
    pub tags: Vec<String>,
    pub index: i32,
    #[serde(default = "default_frequency_weight_pairs")]
    pub freq_weight_pairs: Vec<FrequencyWeightPair>,
    #[serde(default)]
    pub frequency: f64,
    #[serde(default)]
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
            image_path: source.clone(),
            ib: 0.0,
            source,
            source_tag: String::new(),
            tags: default_image_tags(),
            index: 0,
            freq_weight_pairs: default_frequency_weight_pairs(),
            frequency: 0.0,
            weight: 0.0,
            created_at: ts,
            updated_at: ts,
        }
    }
}

pub fn default_image_tags() -> Vec<String> {
    Vec::new()
}

pub fn normalize_image_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim().to_string();
        if tag.is_empty() || normalized.iter().any(|existing| existing == &tag) {
            continue;
        }
        normalized.push(tag);
        if normalized.len() == 2 {
            break;
        }
    }
    normalized
}

pub fn primary_tag(tags: &[String]) -> &str {
    tags.first().map(String::as_str).unwrap_or("")
}

pub fn secondary_tag(tags: &[String]) -> &str {
    tags.get(1).map(String::as_str).unwrap_or("")
}

pub fn tags_label(tags: &[String]) -> String {
    if tags.is_empty() {
        "No tag".to_string()
    } else {
        tags.join(" + ")
    }
}

pub fn default_frequency_weight_pairs() -> Vec<FrequencyWeightPair> {
    vec![
        FrequencyWeightPair::blank(),
        FrequencyWeightPair::blank(),
        FrequencyWeightPair::blank(),
    ]
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TagDefinition {
    pub name: String,
    pub hue: f64,
}

pub const MAX_TAGS: usize = 24;

pub fn default_tag_definitions() -> Vec<TagDefinition> {
    [15.0, 50.0, 85.0, 120.0, 155.0, 190.0, 225.0, 260.0, 295.0, 330.0]
        .iter()
        .enumerate()
        .map(|(i, hue)| TagDefinition {
            name: format!("tag{}", i + 1),
            hue: *hue,
        })
        .collect()
}

pub fn oklch_from_hue(hue: f64) -> String {
    let h = hue.rem_euclid(360.0);
    format!("oklch(0.72 0.16 {h:.1})")
}

pub fn now_millis() -> i64 {
    js_sys::Date::now() as i64
}
