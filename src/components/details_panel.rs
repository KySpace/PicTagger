use leptos::prelude::*;
use std::collections::HashMap;

use crate::models::{ImageRecord, TagDefinition, oklch_from_hue};

#[component]
pub fn DetailsPanel(
    selected: Memo<Option<ImageRecord>>,
    tags: Memo<Vec<TagDefinition>>,
    on_update: Callback<(String, String)>,
    on_delete: Callback<()>,
) -> impl IntoView {
    let show_preview_modal = RwSignal::new(false);
    let has_selected = move || selected.get().is_some();
    let tag_color_map = Memo::new(move |_| {
        tags.get()
            .into_iter()
            .map(|t| (t.name, oklch_from_hue(t.hue)))
            .collect::<HashMap<_, _>>()
    });
    let selected_tag_color = move || {
        selected
            .get()
            .and_then(|item| tag_color_map.get().get(&item.tag).cloned())
            .unwrap_or_else(|| "transparent".to_string())
    };

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
                                "Tag"
                                <div class="tag-input-row">
                                    <span
                                        class="tag-color-swatch"
                                        style=move || format!("background:{};", selected_tag_color())
                                    ></span>
                                    <input
                                        type="text"
                                        list="tag-options"
                                        prop:value=move || {
                                            selected.get().map(|item| item.tag).unwrap_or_default()
                                        }
                                        on:input=move |ev| on_update.run(("tag".to_string(), event_target_value(&ev)))
                                    />
                                </div>
                                <datalist id="tag-options">
                                    <For
                                        each=move || tags.get()
                                        key=|t| t.name.clone()
                                        children=move |t| view! { <option value=t.name></option> }
                                    />
                                </datalist>
                            </label>
                            <label>
                                "Source"
                                <input
                                    type="text"
                                    prop:value=move || {
                                        selected.get().map(|item| item.source).unwrap_or_default()
                                    }
                                    on:input=move |ev| on_update.run(("source".to_string(), event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Source Tag"
                                <input
                                    type="text"
                                    prop:value=move || {
                                        selected.get().map(|item| item.source_tag).unwrap_or_default()
                                    }
                                    on:input=move |ev| on_update.run(("source_tag".to_string(), event_target_value(&ev)))
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
                                    on:input=move |ev| on_update.run(("ib".to_string(), event_target_value(&ev)))
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
                                    on:input=move |ev| on_update.run(("index".to_string(), event_target_value(&ev)))
                                />
                            </label>
                            <label>
                                "Frequency / Weight"
                                <div class="pair-list">
                                    <For
                                        each=move || {
                                            selected
                                                .get()
                                                .map(|item| {
                                                    item.freq_weight_pairs
                                                        .into_iter()
                                                        .enumerate()
                                                        .collect::<Vec<_>>()
                                                })
                                                .unwrap_or_default()
                                        }
                                        key=|(index, _)| *index
                                        children=move |(index, pair)| {
                                            view! {
                                                <div class="pair-row">
                                                    <input
                                                        type="text"
                                                        inputmode="decimal"
                                                        placeholder="frequency"
                                                        prop:value=pair.frequency.map(|v| v.to_string()).unwrap_or_default()
                                                        on:change=move |ev| {
                                                            on_update.run((format!("pair_frequency:{index}"), event_target_value(&ev)))
                                                        }
                                                    />
                                                    <input
                                                        type="text"
                                                        inputmode="decimal"
                                                        placeholder="weight"
                                                        prop:value=pair.weight.map(|v| v.to_string()).unwrap_or_default()
                                                        on:change=move |ev| {
                                                            on_update.run((format!("pair_weight:{index}"), event_target_value(&ev)))
                                                        }
                                                    />
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            </label>
                            <div class="pair-actions">
                                <button on:click=move |_| on_update.run(("add_pair".to_string(), String::new()))>
                                    "Add Pair"
                                </button>
                                <button on:click=move |_| on_update.run(("clear_pairs".to_string(), String::new()))>
                                    "Clear Pairs"
                                </button>
                            </div>
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
