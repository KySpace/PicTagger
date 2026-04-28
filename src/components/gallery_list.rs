use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{ImageRecord, TagDefinition, oklch_from_hue, primary_tag, tags_label};

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
                        let image_data = item.image_data;
                        let source = item.source;
                        let source_tag = item.source_tag;
                        let tags_label_text = tags_label(&item.tags);
                        let primary_tag_name = primary_tag(&item.tags).to_string();
                        let ib = item.ib;
                        let index = item.index;
                        let active_pair_count = item
                            .freq_weight_pairs
                            .iter()
                            .filter(|pair| pair.frequency.is_some())
                            .count();
                        let has_frequency = active_pair_count > 0;
                        let tag_for_style = primary_tag_name.clone();
                        let card_tag_color = move || {
                            tag_color_map
                                .get()
                                .get(&tag_for_style)
                                .cloned()
                                .unwrap_or_else(|| "transparent".to_string())
                        };
                        let tag_for_swatch = primary_tag_name.clone();
                        let swatch_tag_color = move || {
                            tag_color_map
                                .get()
                                .get(&tag_for_swatch)
                                .cloned()
                                .unwrap_or_else(|| "transparent".to_string())
                        };
                        let is_selected = move || selected_id.get() == Some(id);
                        view! {
                            <article
                                id=format!("gallery-item-{id}")
                                class=move || {
                                    let mut class_name = if is_selected() { "gallery-item selected" } else { "gallery-item" }.to_string();
                                    if !has_frequency {
                                        class_name.push_str(" no-frequency");
                                    }
                                    class_name
                                }
                                style=move || {
                                    if has_frequency {
                                        format!("--item-tag-color: {};", card_tag_color())
                                    } else {
                                        "--item-tag-color: transparent;".to_string()
                                    }
                                }
                                on:click=move |_| on_select.run(id)
                            >
                                <img src=image_data alt="gallery item" loading="lazy" />
                                <div class="gallery-meta">
                                    <p class="source">{source}</p>
                                    <p class="gallery-tag-line">
                                        <span class="tag-color-swatch" style=move || format!("background:{};", swatch_tag_color())></span>
                                        <span>{format!("tags: {tags_label_text}")}</span>
                                    </p>
                                    <p>{format!("source_tag: {source_tag}")}</p>
                                    <p>{format!("IB: {ib:.3}  pairs: {active_pair_count}")}</p>
                                    <p>{format!("index: {index}")}</p>
                                </div>
                            </article>
                        }
                    }
                />
            </div>
            <div class="gallery-actions">
                <button class="danger" on:click=move |_| on_request_delete_all.run(())>
                    "Clear Gallery"
                </button>
            </div>
        </section>
    }
}
