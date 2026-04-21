use leptos::prelude::*;

use crate::models::ImageRecord;

#[component]
pub fn DetailsPanel(
    selected: Memo<Option<ImageRecord>>,
    on_update: Callback<(&'static str, String)>,
    on_delete: Callback<()>,
) -> impl IntoView {
    view! {
        <aside class="details-panel">
            <h2>"Details"</h2>
            {move || match selected.get() {
                Some(item) => {
                    view! {
                        <div class="details-content">
                            <img src=item.image_data.clone() alt="selected preview" />
                            <label>
                                "Source"
                                <input
                                    type="text"
                                    prop:value=item.source
                                    on:input=move |ev| on_update.run(("source", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "IB"
                                <input
                                    type="number"
                                    step="any"
                                    prop:value=item.ib.to_string()
                                    on:input=move |ev| on_update.run(("ib", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Index"
                                <input
                                    type="number"
                                    step="1"
                                    prop:value=item.index.to_string()
                                    on:input=move |ev| on_update.run(("index", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Frequency"
                                <input
                                    type="number"
                                    step="any"
                                    prop:value=item.frequency.to_string()
                                    on:input=move |ev| on_update.run(("frequency", event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Weight"
                                <input
                                    type="number"
                                    step="any"
                                    prop:value=item.weight.to_string()
                                    on:input=move |ev| on_update.run(("weight", event_target_value(&ev)))
                                />
                            </label>
                            <button class="danger" on:click=move |_| on_delete.run(())>
                                "Delete Selected"
                            </button>
                        </div>
                    }
                        .into_any()
                }
                None => {
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
