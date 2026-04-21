use gloo_file::futures::read_as_data_url;
use leptos::ev;
use leptos::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, ScrollBehavior, ScrollIntoViewOptions};

use crate::components::details_panel::DetailsPanel;
use crate::components::filter_bar::IbFilterBar;
use crate::components::gallery_list::GalleryList;
use crate::components::scatter_plot::ScatterPlot;
use crate::components::tag_editor::TagEditor;
use crate::models::{ImageRecord, default_tag_definitions, now_millis};
use crate::storage::{clear_records, load_records, load_tags, save_records, save_tags};
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

    Effect::new(move |_| {
        save_records(&images.get());
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

    let update_selected = move |field: &'static str, value: String| {
        let Some(id) = selected_id.get() else {
            return;
        };
        images.update(|list| {
            if let Some(item) = list.iter_mut().find(|x| x.id == id) {
                let mut changed = false;
                match field {
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
                            <TagEditor tags=tags />
                        }
                            .into_any()
                    }
                }}
            </section>

            <section class="content-grid">
                <GalleryList
                    images=filtered_images
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
