use leptos::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{ImageRecord, TagDefinition, oklch_from_hue};

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
                    key=|item| item.id
                    children=move |item| {
                        let id = item.id;
                        let has_frequency = item.freq_weight_pairs.iter().any(|pair| pair.frequency.is_some());
                        let tag_color = tag_color_map
                            .get()
                            .get(&item.tag)
                            .cloned()
                            .unwrap_or_else(|| "transparent".to_string());
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
                                style=if has_frequency {
                                    format!("--item-tag-color: {};", tag_color)
                                } else {
                                    "--item-tag-color: transparent;".to_string()
                                }
                                on:click=move |_| on_select.run(id)
                            >
                                <img src=item.image_data alt="gallery item" loading="lazy" />
                                <div class="gallery-meta">
                                    <p class="source">{item.source.clone()}</p>
                                    <p>{format!("tag: {}", item.tag)}</p>
                                    <p>{format!("source_tag: {}", item.source_tag)}</p>
                                    <p>{format!("IB: {:.3}  pairs: {}", item.ib, item.freq_weight_pairs.iter().filter(|pair| pair.frequency.is_some()).count())}</p>
                                    <p>{format!("index: {}", item.index)}</p>
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
