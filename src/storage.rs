use std::io::{Cursor, Read, Write};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zip::write::SimpleFileOptions;

use crate::models::{
    FrequencyWeightPair, ImageRecord, TagDefinition, default_frequency_weight_pairs,
    default_image_tags, default_tag_definitions, normalize_image_tags,
};

const STORAGE_KEY: &str = "pictagger.gallery.v1";
const TAGS_STORAGE_KEY: &str = "pictagger.tags.v1";

#[derive(Clone, Serialize, Deserialize)]
pub struct CacheExport {
    pub version: u32,
    pub tags: Vec<TagDefinition>,
    pub images: Vec<CacheImageRecord>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CacheImageRecord {
    pub id: Uuid,
    pub image_path: String,
    pub ib: f64,
    pub source: String,
    pub source_tag: String,
    #[serde(default = "default_image_tags")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing)]
    pub tag: String,
    pub index: i32,
    pub freq_weight_pairs: Vec<FrequencyWeightPair>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
struct StoredImageRecord {
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
    #[serde(default)]
    pub tag: String,
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

fn current_or_legacy_tags(tags: Vec<String>, legacy_tag: String) -> Vec<String> {
    let normalized = normalize_image_tags(tags);
    if normalized.is_empty() {
        normalize_image_tags(vec![legacy_tag])
    } else {
        normalized
    }
}

fn stored_record_into_image(record: StoredImageRecord) -> ImageRecord {
    ImageRecord {
        id: record.id,
        image_data: record.image_data,
        image_path: record.image_path,
        ib: record.ib,
        source: record.source,
        source_tag: record.source_tag,
        tags: current_or_legacy_tags(record.tags, record.tag),
        index: record.index,
        freq_weight_pairs: if record.freq_weight_pairs.is_empty() {
            default_frequency_weight_pairs()
        } else {
            record.freq_weight_pairs
        },
        frequency: record.frequency,
        weight: record.weight,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

impl CacheExport {
    pub fn into_state(self) -> (Vec<ImageRecord>, Vec<TagDefinition>) {
        let images = self
            .images
            .into_iter()
            .map(|record| ImageRecord {
                id: record.id,
                image_data: record.image_path.clone(),
                image_path: record.image_path,
                ib: record.ib,
                source: record.source,
                source_tag: record.source_tag,
                tags: current_or_legacy_tags(record.tags, record.tag),
                index: record.index,
                freq_weight_pairs: if record.freq_weight_pairs.is_empty() {
                    default_frequency_weight_pairs()
                } else {
                    record.freq_weight_pairs
                },
                frequency: 0.0,
                weight: 0.0,
                created_at: record.created_at,
                updated_at: record.updated_at,
            })
            .collect();
        (images, self.tags)
    }
}

fn sanitize_path_part(raw: &str, fallback: &str) -> String {
    let cleaned = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if cleaned.is_empty() {
        fallback.to_string()
    } else {
        cleaned
    }
}

fn extension_from_mime(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/svg+xml" => "svg",
        _ => "img",
    }
}

fn mime_from_path(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default().to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn decode_data_url(data_url: &str) -> Option<(String, Vec<u8>)> {
    let (header, payload) = data_url.split_once(',')?;
    let mime = header
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(';').map(|(mime, _)| mime.to_string()))
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let bytes = if header.ends_with(";base64") {
        BASE64.decode(payload).ok()?
    } else {
        payload.as_bytes().to_vec()
    };
    Some((mime, bytes))
}

fn encode_data_url(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", BASE64.encode(bytes))
}

pub fn load_records() -> Vec<ImageRecord> {
    let Some(storage) = web_storage() else {
        return Vec::new();
    };
    let Ok(Some(raw)) = storage.get_item(STORAGE_KEY) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<StoredImageRecord>>(&raw)
        .unwrap_or_default()
        .into_iter()
        .map(stored_record_into_image)
        .collect()
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

pub fn import_cache_yaml(raw: &str) -> Result<(Vec<ImageRecord>, Vec<TagDefinition>), serde_yaml::Error> {
    let cache = serde_yaml::from_str::<CacheExport>(raw)?;
    Ok(cache.into_state())
}

pub fn export_cache_zip(images: &[ImageRecord], tags: &[TagDefinition]) -> Result<Vec<u8>, String> {
    let mut archive_images = Vec::new();
    let mut zip_bytes = Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(&mut zip_bytes);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for (position, record) in images.iter().enumerate() {
        let mut cache_record = CacheImageRecord {
            id: record.id,
            image_path: record.image_path.clone(),
            ib: record.ib,
            source: record.source.clone(),
            source_tag: record.source_tag.clone(),
            tags: record.tags.clone(),
            tag: String::new(),
            index: record.index,
            freq_weight_pairs: record.freq_weight_pairs.clone(),
            created_at: record.created_at,
            updated_at: record.updated_at,
        };

        if let Some((mime, bytes)) = decode_data_url(&record.image_data) {
            let folder = sanitize_path_part(&record.source_tag, "untagged");
            let stem = sanitize_path_part(&record.source, "image");
            let ext = extension_from_mime(&mime);
            let path = format!("{folder}/{position:04}_{stem}_{}.{}", record.id, ext);
            writer
                .start_file(path.clone(), options)
                .map_err(|err| err.to_string())?;
            writer.write_all(&bytes).map_err(|err| err.to_string())?;
            cache_record.image_path = path;
        }

        archive_images.push(cache_record);
    }

    let cache = CacheExport {
        version: 1,
        tags: tags.to_vec(),
        images: archive_images,
    };
    let yaml = serde_yaml::to_string(&cache).map_err(|err| err.to_string())?;
    writer
        .start_file("cache.yaml", options)
        .map_err(|err| err.to_string())?;
    writer
        .write_all(yaml.as_bytes())
        .map_err(|err| err.to_string())?;
    writer.finish().map_err(|err| err.to_string())?;
    Ok(zip_bytes.into_inner())
}

pub fn import_cache_zip(bytes: &[u8]) -> Result<(Vec<ImageRecord>, Vec<TagDefinition>), String> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|err| err.to_string())?;
    let mut yaml = String::new();
    archive
        .by_name("cache.yaml")
        .map_err(|err| err.to_string())?
        .read_to_string(&mut yaml)
        .map_err(|err| err.to_string())?;
    let cache = serde_yaml::from_str::<CacheExport>(&yaml).map_err(|err| err.to_string())?;
    let tags = cache.tags.clone();
    let mut images = Vec::new();

    for record in cache.images {
        let image_data = if record.image_path.is_empty() {
            String::new()
        } else if let Ok(mut file) = archive.by_name(&record.image_path) {
            let mut image_bytes = Vec::new();
            file.read_to_end(&mut image_bytes)
                .map_err(|err| err.to_string())?;
            encode_data_url(mime_from_path(&record.image_path), &image_bytes)
        } else {
            record.image_path.clone()
        };

        images.push(ImageRecord {
            id: record.id,
            image_data,
            image_path: record.image_path,
            ib: record.ib,
            source: record.source,
            source_tag: record.source_tag,
            tags: current_or_legacy_tags(record.tags, record.tag),
            index: record.index,
            freq_weight_pairs: if record.freq_weight_pairs.is_empty() {
                default_frequency_weight_pairs()
            } else {
                record.freq_weight_pairs
            },
            frequency: 0.0,
            weight: 0.0,
            created_at: record.created_at,
            updated_at: record.updated_at,
        });
    }

    Ok((images, tags))
}

fn web_storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
}
