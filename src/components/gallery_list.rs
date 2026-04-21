use leptos::prelude::*;
use uuid::Uuid;

use crate::models::ImageRecord;

#[component]
pub fn GalleryList(
    images: Memo<Vec<ImageRecord>>,
    selected_id: RwSignal<Option<Uuid>>,
    on_select: Callback<Uuid>,
    on_request_delete_all: Callback<()>,
) -> impl IntoView {
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
                        let is_selected = move || selected_id.get() == Some(id);
                        view! {
                            <article
                                id=format!("gallery-item-{id}")
                                class=move || {
                                    if is_selected() { "gallery-item selected" } else { "gallery-item" }
                                }
                                on:click=move |_| on_select.run(id)
                            >
                                <img src=item.image_data alt="gallery item" loading="lazy" />
                                <div class="gallery-meta">
                                    <p class="source">{item.source.clone()}</p>
                                    <p>{format!("tag: {}", item.tag)}</p>
                                    <p>{format!("source_tag: {}", item.source_tag)}</p>
                                    <p>{format!("IB: {:.3}  freq: {:.3}", item.ib, item.frequency)}</p>
                                    <p>{format!("index: {}  weight: {:.3}", item.index, item.weight)}</p>
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
