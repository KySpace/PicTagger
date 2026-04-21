use leptos::prelude::*;

use crate::models::{MAX_TAGS, TagDefinition, oklch_from_hue};

#[component]
pub fn TagEditor(tags: RwSignal<Vec<TagDefinition>>) -> impl IntoView {
    let new_tag_name = RwSignal::new(String::new());
    let new_tag_hue = RwSignal::new("15".to_string());

    let add_tag = move |_| {
        let name = new_tag_name.get().trim().to_string();
        if name.is_empty() {
            return;
        }
        let hue = new_tag_hue
            .get()
            .trim()
            .parse::<f64>()
            .ok()
            .map(|v| v.clamp(0.0, 360.0))
            .unwrap_or(15.0);
        tags.update(|list| {
            if list.len() >= MAX_TAGS || list.iter().any(|t| t.name == name) {
                return;
            }
            list.push(TagDefinition { name, hue });
        });
        new_tag_name.set(String::new());
    };

    view! {
        <section class="tag-editor-panel">
            <h2>"Tag Editor"</h2>
            <p>{move || format!("{} / {} tags", tags.get().len(), MAX_TAGS)}</p>
            <div class="tag-add-row">
                <label>
                    "Tag Name"
                    <input
                        type="text"
                        placeholder="new tag"
                        prop:value=move || new_tag_name.get()
                        on:input=move |ev| new_tag_name.set(event_target_value(&ev))
                    />
                </label>
                <label>
                    "Hue"
                    <input
                        type="text"
                        inputmode="decimal"
                        placeholder="0..360"
                        prop:value=move || new_tag_hue.get()
                        on:input=move |ev| new_tag_hue.set(event_target_value(&ev))
                    />
                </label>
                <button on:click=add_tag disabled=move || tags.get().len() >= MAX_TAGS>
                    "Add Tag"
                </button>
            </div>

            <div class="tag-list">
                <For
                    each=move || tags.get()
                    key=|t| t.name.clone()
                    children=move |tag| {
                        let name_for_hue = tag.name.clone();
                        let name_for_delete = tag.name.clone();
                        view! {
                            <article class="tag-row">
                                <span
                                    class="tag-chip"
                                    style=format!("background:{};", oklch_from_hue(tag.hue))
                                    title=move || format!("OKLCH hue {:.1}", tag.hue)
                                ></span>
                                <p class="tag-name">{tag.name.clone()}</p>
                                <label>
                                    "Hue"
                                    <input
                                        type="range"
                                        min="0"
                                        max="360"
                                        step="1"
                                        prop:value=tag.hue.to_string()
                                        on:input=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                                tags.update(|list| {
                                                    if let Some(found) = list.iter_mut().find(|t| t.name == name_for_hue) {
                                                        found.hue = v.clamp(0.0, 360.0);
                                                    }
                                                });
                                            }
                                        }
                                    />
                                </label>
                                <span class="hue-value">{format!("{:.0}", tag.hue)}</span>
                                <button
                                    class="danger"
                                    on:click=move |_| {
                                        tags.update(|list| list.retain(|t| t.name != name_for_delete));
                                    }
                                >
                                    "Delete"
                                </button>
                            </article>
                        }
                    }
                />
            </div>
        </section>
    }
}
