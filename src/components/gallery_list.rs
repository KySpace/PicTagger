use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{ImageRecord, TagDefinition, oklch_from_hue};

fn fallback_item() -> ImageRecord {
    ImageRecord::new(String::new(), String::new())
}

#[component]
pub fn GalleryList(
    images: Memo<Vec<ImageRecord>>,
    tags: Memo<Vec<TagDefinition>>,
    selected_id: RwSignal<Option<Uuid>>,
    on_select: Callback<Uuid>,
    on_request_delete_all: Callback<()>,
) -> impl IntoView {
    let tag_color_map = Memo::new(move |_| {
        tags.get()
            .into_iter()
            .map(|t| (t.name, oklch_from_hue(t.hue)))
            .collect::<HashMap<_, _>>()
    });

    view! {
        <section class="gallery-panel">
            <div class="gallery-title">
                <h2>"Gallery"</h2>
                <span>{move || format!("{} items", images.get().len())}</span>
            </div>
            <div class="gallery-list">
                <For
                    each=move || images.get()
                    key=|item| (item.id, item.updated_at)
                    children=move |item| {
                        let id = item.id;
                        let current_item = move || {
                            images
                                .get()
                                .into_iter()
                                .find(|candidate| candidate.id == id)
                                .unwrap_or_else(fallback_item)
                        };
                        let has_frequency = move || {
                            current_item()
                                .freq_weight_pairs
                                .iter()
                                .any(|pair| pair.frequency.is_some())
                        };
                        let tag_color = move || {
                            let item_tag = current_item().tag;
                            tag_color_map
                                .get()
                                .get(&item_tag)
                                .cloned()
                                .unwrap_or_else(|| "transparent".to_string())
                        };
                        let is_selected = move || selected_id.get() == Some(id);
                        view! {
                            <article
                                id=format!("gallery-item-{id}")
                                class=move || {
                                    let mut class_name = if is_selected() { "gallery-item selected" } else { "gallery-item" }.to_string();
                                    if !has_frequency() {
                                        class_name.push_str(" no-frequency");
                                    }
                                    class_name
                                }
                                style=move || {
                                    if has_frequency() {
                                        format!("--item-tag-color: {};", tag_color())
                                    } else {
                                        "--item-tag-color: transparent;".to_string()
                                    }
                                }
                                on:click=move |_| on_select.run(id)
                            >
                                <img src=move || current_item().image_data alt="gallery item" loading="lazy" />
                                <div class="gallery-meta">
                                    <p class="source">{move || current_item().source}</p>
                                    <p class="gallery-tag-line">
                                        <span class="tag-color-swatch" style=move || format!("background:{};", tag_color())></span>
                                        <span>{move || format!("tag: {}", current_item().tag)}</span>
                                    </p>
                                    <p>{move || format!("source_tag: {}", current_item().source_tag)}</p>
                                    <p>{move || {
                                        let item = current_item();
                                        format!(
                                            "IB: {:.3}  pairs: {}",
                                            item.ib,
                                            item.freq_weight_pairs.iter().filter(|pair| pair.frequency.is_some()).count()
                                        )
                                    }}</p>
                                    <p>{move || format!("index: {}", current_item().index)}</p>
                                </div>
                            </article>
                        }
                    }
                />
            </div>
            <div class="gallery-actions">
                <button class="danger" on:click=move |_| on_request_delete_all.run(())>
                    "Delete All"
                </button>
            </div>
        </section>
    }
}
