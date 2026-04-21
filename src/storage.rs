use crate::models::{ImageRecord, TagDefinition, default_tag_definitions};

const STORAGE_KEY: &str = "pictagger.gallery.v1";
const TAGS_STORAGE_KEY: &str = "pictagger.tags.v1";

pub fn load_records() -> Vec<ImageRecord> {
    let Some(storage) = web_storage() else {
        return Vec::new();
    };
    let Ok(Some(raw)) = storage.get_item(STORAGE_KEY) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_records(records: &[ImageRecord]) {
    let Some(storage) = web_storage() else {
        return;
    };
    let Ok(payload) = serde_json::to_string(records) else {
        return;
    };
    let _ = storage.set_item(STORAGE_KEY, &payload);
}

pub fn clear_records() {
    if let Some(storage) = web_storage() {
        let _ = storage.remove_item(STORAGE_KEY);
        let _ = storage.remove_item(TAGS_STORAGE_KEY);
    }
}

pub fn load_tags() -> Vec<TagDefinition> {
    let Some(storage) = web_storage() else {
        return default_tag_definitions();
    };
    let Ok(Some(raw)) = storage.get_item(TAGS_STORAGE_KEY) else {
        return default_tag_definitions();
    };
    serde_json::from_str(&raw).unwrap_or_else(|_| default_tag_definitions())
}

pub fn save_tags(tags: &[TagDefinition]) {
    let Some(storage) = web_storage() else {
        return;
    };
    let Ok(payload) = serde_json::to_string(tags) else {
        return;
    };
    let _ = storage.set_item(TAGS_STORAGE_KEY, &payload);
}

fn web_storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
}
