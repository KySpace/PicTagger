use leptos::prelude::*;

use crate::models::ImageRecord;

#[component]
pub fn DetailsPanel(
    selected: Memo<Option<ImageRecord>>,
    on_update: Callback<(&'static str, String)>,
    on_delete: Callback<()>,
) -> impl IntoView {
    let show_preview_modal = RwSignal::new(false);
    let has_selected = move || selected.get().is_some();

    view! {
        <aside class="details-panel">
            <h2>"Details"</h2>
            {move || {
                if has_selected() {
                    view! {
                        <div class="details-content">
                            <img
                                src=move || {
                                    selected
                                        .get()
                                        .map(|item| item.image_data)
                                        .unwrap_or_default()
                                }
                                alt="selected preview"
                                title="Double click to enlarge"
                                on:dblclick=move |_| show_preview_modal.set(true)
                            />
                            <label>
                                "Source"
                                <input
                                    type="text"
                                    prop:value=move || {
                                        selected.get().map(|item| item.source).unwrap_or_default()
                                    }
                                    on:input=move |ev| on_update.run(("source", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Source Tag"
                                <input
                                    type="text"
                                    prop:value=move || {
                                        selected.get().map(|item| item.source_tag).unwrap_or_default()
                                    }
                                    on:input=move |ev| on_update.run(("source_tag", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "IB"
                                <input
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || {
                                        selected
                                            .get()
                                            .map(|item| item.ib.to_string())
                                            .unwrap_or_default()
                                    }
                                    on:input=move |ev| on_update.run(("ib", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Index"
                                <input
                                    type="text"
                                    inputmode="numeric"
                                    prop:value=move || {
                                        selected
                                            .get()
                                            .map(|item| item.index.to_string())
                                            .unwrap_or_default()
                                    }
                                    on:input=move |ev| on_update.run(("index", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Frequency"
                                <input
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || {
                                        selected
                                            .get()
                                            .map(|item| item.frequency.to_string())
                                            .unwrap_or_default()
                                    }
                                    on:input=move |ev| on_update.run(("frequency", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Weight"
                                <input
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || {
                                        selected
                                            .get()
                                            .map(|item| item.weight.to_string())
                                            .unwrap_or_default()
                                    }
                                    on:input=move |ev| on_update.run(("weight", event_target_value(&ev)))
                                />
                            </label>
                            <button class="danger" on:click=move |_| on_delete.run(())>
                                "Delete Selected"
                            </button>
                        </div>
                        {move || {
                            if show_preview_modal.get() {
                                view! {
                                    <div
                                        class="modal-backdrop image-preview-backdrop"
                                        on:click=move |_| show_preview_modal.set(false)
                                    >
                                        <div class="image-preview-card" on:click=move |ev| ev.stop_propagation()>
                                            <img
                                                src=move || {
                                                    selected
                                                        .get()
                                                        .map(|item| item.image_data)
                                                        .unwrap_or_default()
                                                }
                                                alt="full preview"
                                            />
                                            <div class="modal-actions">
                                                <button on:click=move |_| show_preview_modal.set(false)>"Close"</button>
                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                    }
                        .into_any()
                } else {
                    view! {
                        <div class="details-empty">
                            "Select an image from the gallery or scatter plot."
                        </div>
                    }
                        .into_any()
                }
            }}
        </aside>
    }
}
