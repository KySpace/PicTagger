use gloo_file::futures::{read_as_bytes, read_as_data_url, read_as_text};
use js_sys::{Array, Uint8Array};
use leptos::ev;
use leptos::prelude::*;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    Blob, BlobPropertyBag, HtmlAnchorElement, HtmlInputElement, ScrollBehavior,
    ScrollIntoViewOptions, Url,
};

use crate::components::details_panel::DetailsPanel;
use crate::components::filter_bar::IbFilterBar;
use crate::components::gallery_list::GalleryList;
use crate::components::scatter_plot::ScatterPlot;
use crate::components::tag_editor::TagEditor;
use crate::models::{FrequencyWeightPair, ImageRecord, default_frequency_weight_pairs, default_tag_definitions, now_millis};
use crate::storage::{
    clear_records, export_cache_zip, import_cache_yaml, import_cache_zip, load_records, load_tags,
    save_records, save_tags,
};
use uuid::Uuid;

#[derive(Clone)]
struct PendingUpload {
    name: String,
    file: gloo_file::File,
}

fn extract_first_number(name: &str) -> Option<i32> {
    let mut digits = String::new();
    let mut in_digits = false;
    for ch in name.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            in_digits = true;
        } else if in_digits {
            break;
        }
    }
    if digits.is_empty() {
        None
    } else {
        digits.parse::<i32>().ok()
    }
}

fn infer_unique_indexes(files: &[PendingUpload]) -> Vec<Option<i32>> {
    let mut counts: HashMap<i32, usize> = HashMap::new();
    let mut extracted = Vec::with_capacity(files.len());
    for file in files {
        let value = extract_first_number(&file.name);
        if let Some(idx) = value {
            *counts.entry(idx).or_insert(0) += 1;
        }
        extracted.push(value);
    }
    extracted
        .into_iter()
        .map(|value| value.filter(|idx| counts.get(idx).copied().unwrap_or(0) == 1))
        .collect()
}

fn process_upload_batch(
    files: Vec<PendingUpload>,
    source_tag: String,
    ib: Option<f64>,
    images: RwSignal<Vec<ImageRecord>>,
    selected_id: RwSignal<Option<Uuid>>,
) {
    let inferred_indexes = infer_unique_indexes(&files);
    for (file, index_guess) in files.into_iter().zip(inferred_indexes.into_iter()) {
        let file_name = file.name.clone();
        let gloo_file = file.file;
        let source_tag = source_tag.clone();

        spawn_local(async move {
            if let Ok(data_url) = read_as_data_url(&gloo_file).await {
                let mut record = ImageRecord::new(data_url, file_name);
                record.source_tag = source_tag;
                if let Some(shared_ib) = ib {
                    record.ib = shared_ib;
                }
                if let Some(inferred_index) = index_guess {
                    record.index = inferred_index;
                }
                let id = record.id;
                images.update(|list| list.push(record));
                selected_id.set(Some(id));
            }
        });
    }
}

fn download_binary_file(filename: &str, content: &[u8], mime_type: &str) {
    let parts = Array::new();
    let bytes = Uint8Array::from(content);
    parts.push(&bytes);

    let options = BlobPropertyBag::new();
    options.set_type(mime_type);
    let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &options) else {
        return;
    };
    let Ok(url) = Url::create_object_url_with_blob(&blob) else {
        return;
    };
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        let _ = Url::revoke_object_url(&url);
        return;
    };
    let Ok(element) = document.create_element("a") else {
        let _ = Url::revoke_object_url(&url);
        return;
    };
    let Ok(anchor) = element.dyn_into::<HtmlAnchorElement>() else {
        let _ = Url::revoke_object_url(&url);
        return;
    };
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();
    let _ = Url::revoke_object_url(&url);
}

#[component]
pub fn App() -> impl IntoView {
    let images = RwSignal::new(load_records());
    let tags = RwSignal::new(load_tags());
    let selected_id = RwSignal::new(None::<Uuid>);
    let filter_ib_min = RwSignal::new(None::<f64>);
    let filter_ib_max = RwSignal::new(None::<f64>);
    let hover_id = RwSignal::new(None::<Uuid>);
    let pending_uploads = RwSignal::new(Vec::<PendingUpload>::new());
    let show_batch_modal = RwSignal::new(false);
    let show_delete_all_modal = RwSignal::new(false);
    let batch_source_tag = RwSignal::new(String::new());
    let batch_ib = RwSignal::new(String::new());
    let active_top_tab = RwSignal::new("scatter".to_string());

    let save_records_timer = Rc::new(Cell::new(None::<i32>));
    Effect::new(move |_| {
        let records = images.get();
        let Some(window) = web_sys::window() else {
            save_records(&records);
            return;
        };

        if let Some(handle) = save_records_timer.get() {
            window.clear_timeout_with_handle(handle);
        }

        let timer = Rc::clone(&save_records_timer);
        let callback = Closure::once(move || {
            save_records(&records);
            timer.set(None);
        });

        match window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            300,
        ) {
            Ok(handle) => {
                save_records_timer.set(Some(handle));
                callback.forget();
            }
            Err(_) => save_records(&images.get_untracked()),
        }
    });
    Effect::new(move |_| {
        save_tags(&tags.get());
    });

    let filtered_images = Memo::new(move |_| {
        let min = filter_ib_min.get();
        let max = filter_ib_max.get();
        images
            .get()
            .into_iter()
            .filter(|item| min.is_none_or(|v| item.ib >= v))
            .filter(|item| max.is_none_or(|v| item.ib <= v))
            .collect::<Vec<_>>()
    });

    let on_files_picked = move |ev: ev::Event| {
        let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(file_list) = input.files() else {
            return;
        };

        let mut picked = Vec::new();
        for i in 0..file_list.length() {
            let Some(file) = file_list.get(i) else {
                continue;
            };
            picked.push(PendingUpload {
                name: file.name(),
                file: gloo_file::File::from(file),
            });
        }
        if picked.is_empty() {
            return;
        }

        if picked.len() > 1 {
            pending_uploads.set(picked);
            batch_source_tag.set(String::new());
            batch_ib.set(String::new());
            show_batch_modal.set(true);
        } else {
            process_upload_batch(picked, String::new(), None, images, selected_id);
        }

        input.set_value("");
    };

    let on_export_zip = move |_| {
        if let Ok(payload) = export_cache_zip(&images.get(), &tags.get()) {
            download_binary_file("pictagger-cache.zip", &payload, "application/zip");
        }
    };

    let on_import_cache = move |ev: ev::Event| {
        let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(file_list) = input.files() else {
            return;
        };
        let Some(file) = file_list.get(0) else {
            return;
        };
        let file_name = file.name();
        let cache_file = gloo_file::File::from(file);
        let images = images;
        let tags = tags;
        let selected_id = selected_id;
        let hover_id = hover_id;
        spawn_local(async move {
            let imported = if file_name.to_ascii_lowercase().ends_with(".zip") {
                read_as_bytes(&cache_file)
                    .await
                    .ok()
                    .and_then(|bytes| import_cache_zip(&bytes).ok())
            } else {
                read_as_text(&cache_file)
                    .await
                    .ok()
                    .and_then(|raw| import_cache_yaml(&raw).ok())
            };

            if let Some((imported_images, imported_tags)) = imported {
                    images.set(imported_images);
                    tags.set(if imported_tags.is_empty() {
                        default_tag_definitions()
                    } else {
                        imported_tags
                    });
                    selected_id.set(None);
                    hover_id.set(None);
            }
        });
        input.set_value("");
    };

    let update_selected = move |field: String, value: String| {
        let Some(id) = selected_id.get() else {
            return;
        };
        images.update(|list| {
            if let Some(item) = list.iter_mut().find(|x| x.id == id) {
                let mut changed = false;
                match field.as_str() {
                    "add_pair" => {
                        item.freq_weight_pairs.push(FrequencyWeightPair::blank());
                        changed = true;
                    }
                    "clear_pairs" => {
                        item.freq_weight_pairs = default_frequency_weight_pairs();
                        changed = true;
                    }
                    "tag" => {
                        if item.tag != value {
                            item.tag = value;
                            changed = true;
                        }
                    }
                    "source" => {
                        if item.source != value {
                            item.source = value;
                            changed = true;
                        }
                    }
                    "source_tag" => {
                        if item.source_tag != value {
                            item.source_tag = value;
                            changed = true;
                        }
                    }
                    "ib" => {
                        if let Ok(v) = value.parse::<f64>() {
                            if (item.ib - v).abs() > f64::EPSILON {
                                item.ib = v;
                                changed = true;
                            }
                        }
                    }
                    "index" => {
                        if let Ok(v) = value.parse::<i32>() {
                            if item.index != v {
                                item.index = v;
                                changed = true;
                            }
                        }
                    }
                    "frequency" => {
                        if let Ok(v) = value.parse::<f64>() {
                            if (item.frequency - v).abs() > f64::EPSILON {
                                item.frequency = v;
                                changed = true;
                            }
                        }
                    }
                    "weight" => {
                        if let Ok(v) = value.parse::<f64>() {
                            if (item.weight - v).abs() > f64::EPSILON {
                                item.weight = v;
                                changed = true;
                            }
                        }
                    }
                    _ if field.starts_with("pair_frequency:") => {
                        if let Ok(index) = field["pair_frequency:".len()..].parse::<usize>() {
                            if let Some(pair) = item.freq_weight_pairs.get_mut(index) {
                                let next = if value.trim().is_empty() {
                                    None
                                } else {
                                    value.trim().parse::<f64>().ok()
                                };
                                if pair.frequency != next {
                                    pair.frequency = next;
                                    changed = true;
                                }
                            }
                        }
                    }
                    _ if field.starts_with("pair_weight:") => {
                        if let Ok(index) = field["pair_weight:".len()..].parse::<usize>() {
                            if let Some(pair) = item.freq_weight_pairs.get_mut(index) {
                                let next = if value.trim().is_empty() {
                                    None
                                } else {
                                    value.trim().parse::<f64>().ok()
                                };
                                if pair.weight != next {
                                    pair.weight = next;
                                    changed = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                if changed {
                    item.updated_at = now_millis();
                }
            }
        });
    };

    let on_delete_selected = move || {
        let Some(id) = selected_id.get() else {
            return;
        };
        images.update(|list| list.retain(|x| x.id != id));
        selected_id.set(None);
        hover_id.set(None);
    };

    let on_clear_all = move || {
        images.set(Vec::new());
        tags.set(default_tag_definitions());
        selected_id.set(None);
        hover_id.set(None);
        clear_records();
    };

    let select_and_scroll = move |id: Uuid| {
        selected_id.set(Some(id));
        let element_id = format!("gallery-item-{id}");
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&element_id))
        {
            let options = ScrollIntoViewOptions::new();
            options.set_behavior(ScrollBehavior::Smooth);
            el.scroll_into_view_with_scroll_into_view_options(&options);
        }
    };

    let selected_record = Memo::new(move |_| {
        selected_id
            .get()
            .and_then(|id| images.get().into_iter().find(|item| item.id == id))
    });
    let tags_memo = Memo::new(move |_| tags.get());

    let on_cancel_batch = move |_| {
        pending_uploads.set(Vec::new());
        show_batch_modal.set(false);
        batch_source_tag.set(String::new());
        batch_ib.set(String::new());
    };

    let on_confirm_batch = move |_| {
        let files = pending_uploads.get();
        if files.is_empty() {
            show_batch_modal.set(false);
            return;
        }
        let source_tag = batch_source_tag.get().trim().to_string();
        let ib = batch_ib.get().trim().parse::<f64>().ok();
        process_upload_batch(files, source_tag, ib, images, selected_id);
        pending_uploads.set(Vec::new());
        show_batch_modal.set(false);
        batch_source_tag.set(String::new());
        batch_ib.set(String::new());
    };

    view! {
        <main class="app-shell">
            <header class="toolbar">
                <div class="toolbar-left">
                    <h1>"PicTagger"</h1>
                    <p>"Leptos front-end MVP"</p>
                </div>
                <div class="toolbar-right">
                    <button on:click=on_export_zip>"Export ZIP"</button>
                    <label class="button-like">
                        "Import Cache"
                        <input
                            type="file"
                            accept=".zip,.yaml,.yml,application/zip,application/x-yaml,text/yaml,text/plain"
                            on:change=on_import_cache
                            style="display:none"
                        />
                    </label>
                    <label class="button-like">
                        "Add Images"
                        <input
                            type="file"
                            accept="image/*"
                            multiple
                            on:change=on_files_picked
                            style="display:none"
                        />
                    </label>
                </div>
            </header>

            <IbFilterBar
                filter_ib_min=filter_ib_min
                filter_ib_max=filter_ib_max
            />

            <section class="top-tabs">
                <div class="tab-header">
                    <button
                        class=move || {
                            if active_top_tab.get() == "scatter" { "tab-btn active" } else { "tab-btn" }
                        }
                        on:click=move |_| active_top_tab.set("scatter".to_string())
                    >
                        "Scatter Plot"
                    </button>
                    <button
                        class=move || {
                            if active_top_tab.get() == "tags" { "tab-btn active" } else { "tab-btn" }
                        }
                        on:click=move |_| active_top_tab.set("tags".to_string())
                    >
                        "Tag Editor"
                    </button>
                </div>
                {move || {
                    if active_top_tab.get() == "scatter" {
                        view! {
                            <ScatterPlot
                                images=filtered_images
                                tags=tags_memo
                                selected_id=selected_id
                                hover_id=hover_id
                                on_select=Callback::new(move |id| selected_id.set(Some(id)))
                                on_jump=Callback::new(select_and_scroll)
                            />
                        }
                            .into_any()
                    } else {
                        view! {
                            <TagEditor tags=tags images=images />
                        }
                            .into_any()
                    }
                }}
            </section>

            <section class="content-grid">
                <GalleryList
                    images=filtered_images
                    tags=tags_memo
                    selected_id=selected_id
                    on_select=Callback::new(move |id| selected_id.set(Some(id)))
                    on_request_delete_all=Callback::new(move |_| show_delete_all_modal.set(true))
                />
                <DetailsPanel
                    selected=selected_record
                    tags=tags_memo
                    on_update=Callback::new(move |(field, value)| update_selected(field, value))
                    on_delete=Callback::new(move |_| on_delete_selected())
                />
            </section>

            {move || {
                if show_batch_modal.get() {
                    view! {
                        <div class="modal-backdrop">
                            <div class="modal-card">
                                <h3>"Batch Import Settings"</h3>
                                <p>{move || format!("{} images selected", pending_uploads.get().len())}</p>
                                <label>
                                    "Source Tag"
                                    <input
                                        type="text"
                                        placeholder="e.g. experiment_a"
                                        prop:value=move || batch_source_tag.get()
                                        on:input=move |ev| batch_source_tag.set(event_target_value(&ev))
                                    />
                                </label>
                                <label>
                                    "IB (applies to all selected images)"
                                    <input
                                        type="text"
                                        inputmode="decimal"
                                        placeholder="leave blank to keep default"
                                        prop:value=move || batch_ib.get()
                                        on:input=move |ev| batch_ib.set(event_target_value(&ev))
                                    />
                                </label>
                                <p class="modal-note">
                                    "Index will auto-fill from filename number only when that number is unique in this import batch."
                                </p>
                                <div class="modal-actions">
                                    <button on:click=on_cancel_batch>"Cancel"</button>
                                    <button on:click=on_confirm_batch>"Import"</button>
                                </div>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}

            {move || {
                if show_delete_all_modal.get() {
                    view! {
                        <div class="modal-backdrop">
                            <div class="modal-card">
                                <h3>"Delete All Images?"</h3>
                                <p>"This will permanently remove all gallery records and local saved data."</p>
                                <div class="modal-actions">
                                    <button on:click=move |_| show_delete_all_modal.set(false)>"Cancel"</button>
                                    <button
                                        class="danger"
                                        on:click=move |_| {
                                            show_delete_all_modal.set(false);
                                            on_clear_all();
                                        }
                                    >
                                        "Delete All"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}
        </main>
    }
}
