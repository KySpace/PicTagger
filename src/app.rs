use gloo_file::futures::read_as_data_url;
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, ScrollBehavior, ScrollIntoViewOptions};

use crate::components::details_panel::DetailsPanel;
use crate::components::filter_bar::IbFilterBar;
use crate::components::gallery_list::GalleryList;
use crate::components::scatter_plot::ScatterPlot;
use crate::models::{ImageRecord, now_millis};
use crate::storage::{clear_records, load_records, save_records};
use uuid::Uuid;

#[component]
pub fn App() -> impl IntoView {
    let images = RwSignal::new(load_records());
    let selected_id = RwSignal::new(None::<Uuid>);
    let filter_ib_min = RwSignal::new(None::<f64>);
    let filter_ib_max = RwSignal::new(None::<f64>);
    let hover_id = RwSignal::new(None::<Uuid>);

    Effect::new(move |_| {
        save_records(&images.get());
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

        for i in 0..file_list.length() {
            let Some(file) = file_list.get(i) else {
                continue;
            };
            let file_name = file.name();
            let gloo_file = gloo_file::File::from(file);

            let images = images;
            let selected_id = selected_id;
            spawn_local(async move {
                if let Ok(data_url) = read_as_data_url(&gloo_file).await {
                    let record = ImageRecord::new(data_url, file_name);
                    let id = record.id;
                    images.update(|list| list.push(record));
                    selected_id.set(Some(id));
                }
            });
        }
    };

    let update_selected = move |field: &'static str, value: String| {
        let Some(id) = selected_id.get() else {
            return;
        };
        images.update(|list| {
            if let Some(item) = list.iter_mut().find(|x| x.id == id) {
                match field {
                    "source" => item.source = value,
                    "ib" => {
                        if let Ok(v) = value.parse::<f64>() {
                            item.ib = v;
                        }
                    }
                    "index" => {
                        if let Ok(v) = value.parse::<i32>() {
                            item.index = v;
                        }
                    }
                    "frequency" => {
                        if let Ok(v) = value.parse::<f64>() {
                            item.frequency = v;
                        }
                    }
                    "weight" => {
                        if let Ok(v) = value.parse::<f64>() {
                            item.weight = v;
                        }
                    }
                    _ => {}
                }
                item.updated_at = now_millis();
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
                    <button class="danger" on:click=move |_| on_clear_all()>"Clear All"</button>
                </div>
            </header>

            <IbFilterBar
                filter_ib_min=filter_ib_min
                filter_ib_max=filter_ib_max
            />

            <ScatterPlot
                images=filtered_images
                selected_id=selected_id
                hover_id=hover_id
                on_select=Callback::new(move |id| selected_id.set(Some(id)))
                on_jump=Callback::new(select_and_scroll)
            />

            <section class="content-grid">
                <GalleryList
                    images=filtered_images
                    selected_id=selected_id
                    on_select=Callback::new(move |id| selected_id.set(Some(id)))
                />
                <DetailsPanel
                    selected=selected_record
                    on_update=Callback::new(move |(field, value)| update_selected(field, value))
                    on_delete=Callback::new(move |_| on_delete_selected())
                />
            </section>
        </main>
    }
}
