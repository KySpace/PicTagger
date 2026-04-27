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
    let show_tag_menu = RwSignal::new(false);
    let source_draft = RwSignal::new(String::new());
    let source_tag_draft = RwSignal::new(String::new());
    let ib_draft = RwSignal::new(String::new());
    let index_draft = RwSignal::new(String::new());
    let selected_id = Memo::new(move |_| selected.get().map(|item| item.id));
    let has_selected = move || selected_id.get().is_some();

    Effect::new(move |_| {
        if let Some(item) = selected.get() {
            source_draft.set(item.source);
            source_tag_draft.set(item.source_tag);
            ib_draft.set(item.ib.to_string());
            index_draft.set(item.index.to_string());
        } else {
            source_draft.set(String::new());
            source_tag_draft.set(String::new());
            ib_draft.set(String::new());
            index_draft.set(String::new());
        }
    });

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
                            <div class="details-form">
                                <label>
                                    "Tag"
                                    <div class="tag-select">
                                        <button
                                            type="button"
                                            class="tag-select-button"
                                            on:click=move |_| show_tag_menu.update(|shown| *shown = !*shown)
                                        >
                                            <span
                                                class="tag-color-swatch"
                                                style=move || format!("background:{};", selected_tag_color())
                                            ></span>
                                            <span>
                                                {move || {
                                                    selected
                                                        .get()
                                                        .map(|item| {
                                                            if item.tag.is_empty() {
                                                                "No tag".to_string()
                                                            } else {
                                                                item.tag
                                                            }
                                                        })
                                                        .unwrap_or_else(|| "No tag".to_string())
                                                }}
                                            </span>
                                        </button>
                                        {move || {
                                            if show_tag_menu.get() {
                                                view! {
                                                    <div class="tag-select-menu">
                                                        <button
                                                            type="button"
                                                            class="tag-select-option"
                                                            on:click=move |_| {
                                                                on_update.run(("tag".to_string(), String::new()));
                                                                show_tag_menu.set(false);
                                                            }
                                                        >
                                                            <span class="tag-color-swatch empty"></span>
                                                            <span>"No tag"</span>
                                                        </button>
                                                        <For
                                                            each=move || tags.get()
                                                            key=|t| t.name.clone()
                                                            children=move |t| {
                                                                let name = t.name.clone();
                                                                let color = oklch_from_hue(t.hue);
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        class="tag-select-option"
                                                                        on:click=move |_| {
                                                                            on_update.run(("tag".to_string(), name.clone()));
                                                                            show_tag_menu.set(false);
                                                                        }
                                                                    >
                                                                        <span
                                                                            class="tag-color-swatch"
                                                                            style=format!("background:{};", color)
                                                                        ></span>
                                                                        <span>{t.name}</span>
                                                                    </button>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
                                    </div>
                                </label>
                                <label>
                                    "Source"
                                    <input
                                        type="text"
                                        prop:value=move || source_draft.get()
                                        on:input=move |ev| source_draft.set(event_target_value(&ev))
                                        on:change=move |_| {
                                            on_update.run(("source".to_string(), source_draft.get()));
                                        }
                                    />
                                </label>
                                <label>
                                    "Source Tag"
                                    <input
                                        type="text"
                                        prop:value=move || source_tag_draft.get()
                                        on:input=move |ev| source_tag_draft.set(event_target_value(&ev))
                                        on:change=move |_| {
                                            on_update.run(("source_tag".to_string(), source_tag_draft.get()));
                                        }
                                    />
                                </label>
                                <label>
                                    "IB"
                                    <input
                                        type="text"
                                        inputmode="decimal"
                                        prop:value=move || ib_draft.get()
                                        on:input=move |ev| ib_draft.set(event_target_value(&ev))
                                        on:change=move |_| {
                                            on_update.run(("ib".to_string(), ib_draft.get()));
                                        }
                                    />
                                </label>
                                <label>
                                    "Index"
                                    <input
                                        type="text"
                                        inputmode="numeric"
                                        prop:value=move || index_draft.get()
                                        on:input=move |ev| index_draft.set(event_target_value(&ev))
                                        on:change=move |_| {
                                            on_update.run(("index".to_string(), index_draft.get()));
                                        }
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
                                                        value=pair.frequency.map(|v| v.to_string()).unwrap_or_default()
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            on_update.run((format!("pair_frequency:{index}"), value));
                                                        }
                                                    />
                                                    <input
                                                        type="text"
                                                        inputmode="decimal"
                                                        placeholder="weight"
                                                        value=pair.weight.map(|v| v.to_string()).unwrap_or_default()
                                                        on:change=move |ev| {
                                                            let value = event_target_value(&ev);
                                                            on_update.run((format!("pair_weight:{index}"), value));
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
                            <div class="details-image-pane">
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
                            </div>
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
