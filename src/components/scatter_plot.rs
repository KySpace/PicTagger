use js_sys::Function;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;

use crate::models::{ImageRecord, TagDefinition, primary_tag, tags_label};

const AXIS_LIMITS_STORAGE_KEY: &str = "pictagger.scatter.axis_limits.v1";
const ALL_TAGS_FILTER: &str = "__all__";
const UNTAGGED_FILTER: &str = "__untagged__";

#[wasm_bindgen(module = "/src/plotly_bridge.js")]
extern "C" {
    #[wasm_bindgen(js_name = renderPlotlyScatter)]
    fn render_plotly_scatter(
        element: &web_sys::Element,
        payload_json: &str,
        on_select: &Function,
        on_hover: &Function,
        on_unhover: &Function,
        on_mark: &Function,
    );
}

#[derive(Clone, Serialize, Deserialize)]
struct StoredAxisState {
    x_min_input: String,
    x_max_input: String,
    y_min_input: String,
    y_max_input: String,
    manual_limits: Option<[f64; 4]>,
    #[serde(default = "linear_axis")]
    x_axis_type: String,
    #[serde(default = "linear_axis")]
    y_axis_type: String,
    #[serde(default)]
    threshold_mode: bool,
    #[serde(default)]
    weight_threshold: f64,
    #[serde(default = "all_tags_filter")]
    tag_filter: String,
}

impl Default for StoredAxisState {
    fn default() -> Self {
        Self {
            x_min_input: String::new(),
            x_max_input: String::new(),
            y_min_input: String::new(),
            y_max_input: String::new(),
            manual_limits: None,
            x_axis_type: linear_axis(),
            y_axis_type: linear_axis(),
            threshold_mode: false,
            weight_threshold: 0.0,
            tag_filter: all_tags_filter(),
        }
    }
}

#[derive(Serialize)]
struct PlotPayload {
    points: Vec<PlotPoint>,
    selected_id: Option<String>,
    manual_limits: Option<[f64; 4]>,
    x_axis_type: String,
    y_axis_type: String,
    axis_view_revision: u64,
    threshold_mode: bool,
    weight_threshold: f64,
    mark_mode: bool,
}

#[derive(Clone, PartialEq, Serialize)]
struct PlotPoint {
    id: String,
    pair_index: usize,
    ib: f64,
    frequency: f64,
    weight: f64,
    source: String,
    source_tag: String,
    tags: String,
    color: String,
}

#[derive(Clone, PartialEq)]
struct MarkedPreview {
    id: Uuid,
    pair_index: usize,
    dot_x: f64,
    dot_y: f64,
    left: f64,
    top: f64,
}

#[derive(Clone, PartialEq)]
struct DraggingMarkedPreview {
    id: Uuid,
    pair_index: usize,
    offset_x: f64,
    offset_y: f64,
}

fn linear_axis() -> String {
    "linear".to_string()
}

fn all_tags_filter() -> String {
    ALL_TAGS_FILTER.to_string()
}

fn load_axis_state() -> StoredAxisState {
    let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    else {
        return StoredAxisState::default();
    };
    let Ok(Some(raw)) = storage.get_item(AXIS_LIMITS_STORAGE_KEY) else {
        return StoredAxisState::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_axis_state(state: &StoredAxisState) {
    let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    else {
        return;
    };
    let Ok(raw) = serde_json::to_string(state) else {
        return;
    };
    let _ = storage.set_item(AXIS_LIMITS_STORAGE_KEY, &raw);
}

fn parse_f64_opt(raw: &str) -> Option<f64> {
    raw.trim().parse::<f64>().ok()
}

fn plotly_color_from_hue(hue: f64) -> String {
    let lightness = 0.72;
    let chroma = 0.16;
    let radians = hue.rem_euclid(360.0).to_radians();
    let a = chroma * radians.cos();
    let b = chroma * radians.sin();

    let l = lightness + 0.3963377774 * a + 0.2158037573 * b;
    let m = lightness - 0.1055613458 * a - 0.0638541728 * b;
    let s = lightness - 0.0894841775 * a - 1.2914855480 * b;

    let l = l * l * l;
    let m = m * m * m;
    let s = s * s * s;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    let to_srgb_channel = |value: f64| {
        let value = value.clamp(0.0, 1.0);
        let encoded = if value <= 0.0031308 {
            12.92 * value
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        };
        (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
    };

    format!(
        "#{:02x}{:02x}{:02x}",
        to_srgb_channel(r),
        to_srgb_channel(g),
        to_srgb_channel(b)
    )
}

fn position_hover_card(cursor_x: f64, cursor_y: f64) -> (f64, f64) {
    let (viewport_width, viewport_height) = web_sys::window()
        .map(|window| {
            (
                window
                    .inner_width()
                    .ok()
                    .and_then(|value| value.as_f64())
                    .unwrap_or(1200.0),
                window
                    .inner_height()
                    .ok()
                    .and_then(|value| value.as_f64())
                    .unwrap_or(800.0),
            )
        })
        .unwrap_or((1200.0, 800.0));

    let card_width = 360.0;
    let card_height = 172.0;
    let gap = 14.0;
    let margin = 12.0;

    let mut left = cursor_x + gap;
    if left + card_width > viewport_width - margin {
        left = cursor_x - card_width - gap;
    }

    let mut top = cursor_y + gap;
    if top + card_height > viewport_height - margin {
        top = cursor_y - card_height - gap;
    }

    (
        left.clamp(margin, (viewport_width - card_width - margin).max(margin)),
        top.clamp(margin, (viewport_height - card_height - margin).max(margin)),
    )
}

fn tag_disk_style(tags: &[String], colors: &HashMap<String, String>) -> String {
    match tags {
        [] => "background: transparent;".to_string(),
        [first] => format!(
            "background:{};",
            colors
                .get(first)
                .cloned()
                .unwrap_or_else(|| "transparent".to_string())
        ),
        [first, second, ..] => {
            let first = colors
                .get(first)
                .cloned()
                .unwrap_or_else(|| "transparent".to_string());
            let second = colors
                .get(second)
                .cloned()
                .unwrap_or_else(|| "transparent".to_string());
            format!("background: linear-gradient(90deg, {first} 0 50%, {second} 50% 100%);")
        }
    }
}

#[component]
pub fn ScatterPlot(
    images: Memo<Vec<ImageRecord>>,
    tags: Memo<Vec<TagDefinition>>,
    selected_id: RwSignal<Option<Uuid>>,
    hover_id: RwSignal<Option<Uuid>>,
    on_select: Callback<Uuid>,
    on_jump: Callback<Uuid>,
) -> impl IntoView {
    let initial_axis_state = load_axis_state();
    let show_axis_menu = RwSignal::new(false);
    let x_min_input = RwSignal::new(initial_axis_state.x_min_input);
    let x_max_input = RwSignal::new(initial_axis_state.x_max_input);
    let y_min_input = RwSignal::new(initial_axis_state.y_min_input);
    let y_max_input = RwSignal::new(initial_axis_state.y_max_input);
    let manual_limits = RwSignal::new(
        initial_axis_state
            .manual_limits
            .map(|v| (v[0], v[1], v[2], v[3])),
    );
    let x_axis_type = RwSignal::new(initial_axis_state.x_axis_type);
    let y_axis_type = RwSignal::new(initial_axis_state.y_axis_type);
    let axis_view_revision = RwSignal::new(0_u64);
    let threshold_mode = RwSignal::new(initial_axis_state.threshold_mode);
    let weight_threshold = RwSignal::new(initial_axis_state.weight_threshold.clamp(0.0, 1.0));
    let tag_filter = RwSignal::new(initial_axis_state.tag_filter);
    let mark_mode = RwSignal::new(false);
    let marked_previews = RwSignal::new(Vec::<MarkedPreview>::new());
    let dragging_mark = RwSignal::new(None::<DraggingMarkedPreview>);
    let axis_error = RwSignal::new(String::new());
    let plot_ref = NodeRef::<leptos::html::Div>::new();
    let hovered_pair = RwSignal::new(None::<(Uuid, usize)>);
    let hover_card_hovered = RwSignal::new(false);
    let hover_card_position = RwSignal::new((24.0, 24.0));
    let hover_generation = RwSignal::new(0_u64);

    Effect::new(move |_| {
        let state = StoredAxisState {
            x_min_input: x_min_input.get(),
            x_max_input: x_max_input.get(),
            y_min_input: y_min_input.get(),
            y_max_input: y_max_input.get(),
            manual_limits: manual_limits.get().map(|(x0, x1, y0, y1)| [x0, x1, y0, y1]),
            x_axis_type: x_axis_type.get(),
            y_axis_type: y_axis_type.get(),
            threshold_mode: threshold_mode.get(),
            weight_threshold: weight_threshold.get(),
            tag_filter: tag_filter.get(),
        };
        save_axis_state(&state);
    });

    let tag_color_map = Memo::new(move |_| {
        tags.get()
            .into_iter()
            .map(|t| (t.name, plotly_color_from_hue(t.hue)))
            .collect::<HashMap<_, _>>()
    });

    let tag_options = Memo::new(move |_| {
        tags.get()
            .into_iter()
            .map(|tag| tag.name)
            .collect::<Vec<_>>()
    });

    let plot_points = Memo::new(move |_| {
        let colors = tag_color_map.get();
        let active_tag_filter = tag_filter.get();
        images
            .get()
            .into_iter()
            .filter(|item| match active_tag_filter.as_str() {
                ALL_TAGS_FILTER => true,
                UNTAGGED_FILTER => item.tags.is_empty(),
                tag_name => item.tags.iter().any(|tag| tag == tag_name),
            })
            .flat_map(|item| {
                let color_tag = match active_tag_filter.as_str() {
                    ALL_TAGS_FILTER | UNTAGGED_FILTER => primary_tag(&item.tags),
                    tag_name => tag_name,
                };
                let color = colors
                    .get(color_tag)
                    .cloned()
                    .unwrap_or_else(|| "#95a0ad".to_string());
                let item_tags = tags_label(&item.tags);
                item.freq_weight_pairs
                    .iter()
                    .enumerate()
                    .filter_map(|(pair_index, pair)| {
                        pair.frequency.map(|frequency| PlotPoint {
                            id: item.id.to_string(),
                            pair_index,
                            ib: item.ib,
                            frequency,
                            weight: pair.weight.unwrap_or(0.0),
                            source: item.source.clone(),
                            source_tag: item.source_tag.clone(),
                            tags: item_tags.clone(),
                            color: color.clone(),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    });

    let auto_extents = Memo::new(move |_| {
        let points = plot_points.get();
        if points.is_empty() {
            return (0.0, 1.0, 0.0, 1.0);
        }
        let min_ib = points.iter().map(|x| x.ib).fold(f64::INFINITY, f64::min);
        let max_ib = points
            .iter()
            .map(|x| x.ib)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_f = points
            .iter()
            .map(|x| x.frequency)
            .fold(f64::INFINITY, f64::min);
        let max_f = points
            .iter()
            .map(|x| x.frequency)
            .fold(f64::NEG_INFINITY, f64::max);

        (
            min_ib,
            if min_ib == max_ib { min_ib + 1.0 } else { max_ib },
            min_f,
            if min_f == max_f { min_f + 1.0 } else { max_f },
        )
    });

    let hovered_preview = Memo::new(move |_| {
        hovered_pair.get().and_then(|(id, pair_index)| {
            images
                .get()
                .into_iter()
                .find(|item| item.id == id)
                .and_then(|item| {
                    item.freq_weight_pairs
                        .get(pair_index)
                        .map(|pair| (item.clone(), pair_index, pair.frequency, pair.weight))
                })
        })
    });

    Effect::new(move |_| {
        let Some(element) = plot_ref.get() else {
            return;
        };

        let payload = PlotPayload {
            points: plot_points.get(),
            selected_id: selected_id.get().map(|id| id.to_string()),
            manual_limits: manual_limits.get().map(|(x0, x1, y0, y1)| [x0, x1, y0, y1]),
            x_axis_type: x_axis_type.get(),
            y_axis_type: y_axis_type.get(),
            axis_view_revision: axis_view_revision.get(),
            threshold_mode: threshold_mode.get(),
            weight_threshold: weight_threshold.get(),
            mark_mode: mark_mode.get(),
        };
        let Ok(payload_json) = serde_json::to_string(&payload) else {
            return;
        };

        let select_callback = Closure::wrap(Box::new(move |raw_id: String| {
            if let Ok(id) = Uuid::parse_str(&raw_id) {
                on_select.run(id);
            }
        }) as Box<dyn Fn(String)>);
        let hover_callback = Closure::wrap(Box::new(move |raw_key: String| {
            let mut parts = raw_key.split(':');
            let raw_id = parts.next().unwrap_or_default();
            let raw_pair_index = parts.next().unwrap_or("0");
            let client_x = parts
                .next()
                .and_then(|raw| raw.parse::<f64>().ok())
                .unwrap_or(24.0);
            let client_y = parts
                .next()
                .and_then(|raw| raw.parse::<f64>().ok())
                .unwrap_or(24.0);
            if let Ok(id) = Uuid::parse_str(raw_id) {
                hover_generation.update(|generation| *generation = generation.wrapping_add(1));
                hover_id.set(Some(id));
                hover_card_hovered.set(false);
                hover_card_position.set(position_hover_card(client_x, client_y));
                hovered_pair.set(Some((
                    id,
                    raw_pair_index.parse::<usize>().unwrap_or_default(),
                )));
            }
        }) as Box<dyn Fn(String)>);
        let unhover_callback = Closure::wrap(Box::new(move || {
            let unhover_generation = hover_generation.get_untracked();
            let callback = Closure::once(move || {
                if hover_generation.get_untracked() == unhover_generation
                    && !hover_card_hovered.get_untracked()
                {
                    hover_id.set(None);
                    hovered_pair.set(None);
                }
            });
            if let Some(window) = web_sys::window() {
                if window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        callback.as_ref().unchecked_ref(),
                        260,
                    )
                    .is_ok()
                {
                    callback.forget();
                    return;
                }
            }
            if hover_generation.get_untracked() == unhover_generation
                && !hover_card_hovered.get_untracked()
            {
                hover_id.set(None);
                hovered_pair.set(None);
            }
        }) as Box<dyn Fn()>);
        let mark_callback = Closure::wrap(Box::new(move |raw_key: String| {
            let mut parts = raw_key.split(':');
            let raw_id = parts.next().unwrap_or_default();
            let raw_pair_index = parts.next().unwrap_or("0");
            let dot_x = parts
                .next()
                .and_then(|raw| raw.parse::<f64>().ok())
                .unwrap_or(24.0);
            let dot_y = parts
                .next()
                .and_then(|raw| raw.parse::<f64>().ok())
                .unwrap_or(24.0);
            let client_x = parts
                .next()
                .and_then(|raw| raw.parse::<f64>().ok())
                .unwrap_or(dot_x);
            let client_y = parts
                .next()
                .and_then(|raw| raw.parse::<f64>().ok())
                .unwrap_or(dot_y);
            let Ok(id) = Uuid::parse_str(raw_id) else {
                return;
            };
            let pair_index = raw_pair_index.parse::<usize>().unwrap_or_default();
            let (left, top) = position_hover_card(client_x, client_y);

            marked_previews.update(|marks| {
                if let Some(mark) = marks
                    .iter_mut()
                    .find(|mark| mark.id == id && mark.pair_index == pair_index)
                {
                    mark.dot_x = dot_x;
                    mark.dot_y = dot_y;
                    mark.left = left;
                    mark.top = top;
                } else {
                    marks.push(MarkedPreview {
                        id,
                        pair_index,
                        dot_x,
                        dot_y,
                        left,
                        top,
                    });
                }
            });
        }) as Box<dyn Fn(String)>);

        render_plotly_scatter(
            element.as_ref(),
            &payload_json,
            select_callback.as_ref().unchecked_ref(),
            hover_callback.as_ref().unchecked_ref(),
            unhover_callback.as_ref().unchecked_ref(),
            mark_callback.as_ref().unchecked_ref(),
        );

        select_callback.forget();
        hover_callback.forget();
        unhover_callback.forget();
        mark_callback.forget();
    });

    let apply_axis_limits = move |_| {
        let (auto_x_min, auto_x_max, auto_y_min, auto_y_max) = auto_extents.get();

        let x_min_raw = x_min_input.get();
        let x_max_raw = x_max_input.get();
        let y_min_raw = y_min_input.get();
        let y_max_raw = y_max_input.get();

        let x_min = if x_min_raw.trim().is_empty() {
            Some(auto_x_min)
        } else {
            parse_f64_opt(&x_min_raw)
        };
        let x_max = if x_max_raw.trim().is_empty() {
            Some(auto_x_max)
        } else {
            parse_f64_opt(&x_max_raw)
        };
        let y_min = if y_min_raw.trim().is_empty() {
            Some(auto_y_min)
        } else {
            parse_f64_opt(&y_min_raw)
        };
        let y_max = if y_max_raw.trim().is_empty() {
            Some(auto_y_max)
        } else {
            parse_f64_opt(&y_max_raw)
        };

        match (x_min, x_max, y_min, y_max) {
            (Some(x0), Some(x1), Some(y0), Some(y1)) if x0 < x1 && y0 < y1 => {
                manual_limits.set(Some((x0, x1, y0, y1)));
                axis_view_revision.update(|revision| *revision = revision.wrapping_add(1));
                axis_error.set(format!(
                    "Applied: x[{:.3}, {:.3}], y[{:.3}, {:.3}]",
                    x0, x1, y0, y1
                ));
            }
            _ => axis_error.set(
                "Invalid limits: values must be numeric when provided, and min < max."
                    .to_string(),
            ),
        }
    };

    let reset_axis_limits = move |_| {
        manual_limits.set(None);
        axis_view_revision.update(|revision| *revision = revision.wrapping_add(1));
        axis_error.set("Using auto limits.".to_string());
        x_min_input.set(String::new());
        x_max_input.set(String::new());
        y_min_input.set(String::new());
        y_max_input.set(String::new());
    };

    view! {
        <section class="scatter-panel">
            <div class="scatter-header">
                <h2>"Scattering Plot (frequency vs IB)"</h2>
                <div class="scatter-controls">
                    <label>
                        "X Scale"
                        <select
                            prop:value=move || x_axis_type.get()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                if x_axis_type.get_untracked() != value {
                                    x_axis_type.set(value);
                                    axis_view_revision.update(|revision| *revision = revision.wrapping_add(1));
                                }
                            }
                        >
                            <option value="linear">"Linear"</option>
                            <option value="log">"Log"</option>
                        </select>
                    </label>
                    <label>
                        "Y Scale"
                        <select
                            prop:value=move || y_axis_type.get()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                if y_axis_type.get_untracked() != value {
                                    y_axis_type.set(value);
                                    axis_view_revision.update(|revision| *revision = revision.wrapping_add(1));
                                }
                            }
                        >
                            <option value="linear">"Linear"</option>
                            <option value="log">"Log"</option>
                        </select>
                    </label>
                    <label>
                        "Tag"
                        <select
                            prop:value=move || tag_filter.get()
                            on:change=move |ev| tag_filter.set(event_target_value(&ev))
                        >
                            <option value=ALL_TAGS_FILTER>"All"</option>
                            <option value=UNTAGGED_FILTER>"No tag"</option>
                            {move || {
                                tag_options
                                    .get()
                                    .into_iter()
                                    .map(|name| {
                                        let value = name.clone();
                                        view! {
                                            <option value=value>{name}</option>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </select>
                    </label>
                    <label class="threshold-toggle">
                        "Opaque"
                        <input
                            type="checkbox"
                            prop:checked=move || threshold_mode.get()
                            on:change=move |ev| {
                                threshold_mode.set(event_target_checked(&ev));
                            }
                        />
                    </label>
                    <label class="threshold-toggle">
                        "Mark"
                        <input
                            type="checkbox"
                            prop:checked=move || mark_mode.get()
                            on:change=move |ev| {
                                mark_mode.set(event_target_checked(&ev));
                            }
                        />
                    </label>
                    <label class="threshold-slider">
                        <span>{move || format!("Weight >= {:.2}", weight_threshold.get())}</span>
                        <input
                            type="range"
                            min="0"
                            max="1"
                            step="0.01"
                            prop:value=move || weight_threshold.get().to_string()
                            prop:disabled=move || !threshold_mode.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev)
                                    .parse::<f64>()
                                    .unwrap_or_default()
                                    .clamp(0.0, 1.0);
                                weight_threshold.set(value);
                            }
                        />
                    </label>
                    <button
                        on:click=move |_| {
                            if let Some(id) = selected_id.get() {
                                on_jump.run(id);
                            }
                        }
                    >
                        "Jump Selected"
                    </button>
                    <button on:click=move |_| show_axis_menu.update(|v| *v = !*v)>"Axis Limits"</button>
                </div>
            </div>
            {move || {
                if show_axis_menu.get() {
                    view! {
                        <div class="axis-limit-menu">
                            <label>
                                "X min"
                                <input
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || x_min_input.get()
                                    on:input=move |ev| x_min_input.set(event_target_value(&ev))
                                />
                            </label>
                            <label>
                                "X max"
                                <input
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || x_max_input.get()
                                    on:input=move |ev| x_max_input.set(event_target_value(&ev))
                                />
                            </label>
                            <label>
                                "Y min"
                                <input
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || y_min_input.get()
                                    on:input=move |ev| y_min_input.set(event_target_value(&ev))
                                />
                            </label>
                            <label>
                                "Y max"
                                <input
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || y_max_input.get()
                                    on:input=move |ev| y_max_input.set(event_target_value(&ev))
                                />
                            </label>
                            <div class="axis-limit-actions">
                                <button on:click=apply_axis_limits>"Apply"</button>
                                <button on:click=reset_axis_limits>"Reset"</button>
                            </div>
                            <p class="axis-error">{move || axis_error.get()}</p>
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}
            <div class="scatter-wrap">
                <div node_ref=plot_ref class="plotly-scatter"></div>
            </div>
            {move || {
                let marks = marked_previews.get();
                if marks.is_empty() {
                    ().into_any()
                } else {
                    view! {
                        <svg class="marked-preview-lines" aria-hidden="true">
                            {marks
                                .into_iter()
                                .map(|mark| {
                                    let x2 = mark.left + 180.0;
                                    let y2 = mark.top + 86.0;
                                    view! {
                                        <line
                                            x1=mark.dot_x.to_string()
                                            y1=mark.dot_y.to_string()
                                            x2=x2.to_string()
                                            y2=y2.to_string()
                                        />
                                    }
                                })
                                .collect_view()}
                        </svg>
                    }
                        .into_any()
                }
            }}
            {move || {
                let all_images = images.get();
                marked_previews
                    .get()
                    .into_iter()
                    .filter_map(|mark| {
                        let item = all_images.iter().find(|item| item.id == mark.id)?.clone();
                        let pair = item.freq_weight_pairs.get(mark.pair_index)?;
                        let frequency = pair.frequency;
                        let weight = pair.weight;
                        Some((mark, item, frequency, weight))
                    })
                    .map(|(mark, item, frequency, weight)| {
                        let id = mark.id;
                        let pair_index = mark.pair_index;
                        let tag_disk_style = tag_disk_style(&item.tags, &tag_color_map.get());
                        view! {
                            <div
                                class="plotly-hover-preview marked-preview-card"
                                style=format!("left:{}px; top:{}px;", mark.left, mark.top)
                                on:pointermove=move |ev| {
                                    if let Some(dragging) = dragging_mark.get() {
                                        if dragging.id == id && dragging.pair_index == pair_index {
                                            let next_left = ev.client_x() as f64 - dragging.offset_x;
                                            let next_top = ev.client_y() as f64 - dragging.offset_y;
                                            marked_previews.update(|marks| {
                                                if let Some(mark) = marks
                                                    .iter_mut()
                                                    .find(|mark| mark.id == id && mark.pair_index == pair_index)
                                                {
                                                    mark.left = next_left;
                                                    mark.top = next_top;
                                                }
                                            });
                                        }
                                    }
                                }
                                on:pointerup=move |_| {
                                    if dragging_mark
                                        .get_untracked()
                                        .is_some_and(|dragging| dragging.id == id && dragging.pair_index == pair_index)
                                    {
                                        dragging_mark.set(None);
                                    }
                                }
                                on:pointercancel=move |_| {
                                    if dragging_mark
                                        .get_untracked()
                                        .is_some_and(|dragging| dragging.id == id && dragging.pair_index == pair_index)
                                    {
                                        dragging_mark.set(None);
                                    }
                                }
                            >
                                <img src=item.image_data alt="marked preview" />
                                <div class="plotly-hover-meta">
                                    <div
                                        class="marked-preview-header"
                                        on:pointerdown=move |ev| {
                                            ev.prevent_default();
                                            if let Some(element) = ev
                                                .current_target()
                                                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                                            {
                                                let _ = element.set_pointer_capture(ev.pointer_id());
                                            }
                                            dragging_mark.set(Some(DraggingMarkedPreview {
                                                id,
                                                pair_index,
                                                offset_x: ev.client_x() as f64 - mark.left,
                                                offset_y: ev.client_y() as f64 - mark.top,
                                            }));
                                        }
                                    >
                                        <p class="preview-source">{item.source.clone()}</p>
                                        <button
                                            class="marked-preview-close"
                                            title="Close marked preview"
                                            on:pointerdown=move |ev| ev.stop_propagation()
                                            on:click=move |_| {
                                                marked_previews.update(|marks| {
                                                    marks.retain(|mark| !(mark.id == id && mark.pair_index == pair_index));
                                                });
                                            }
                                        >
                                            "X"
                                        </button>
                                    </div>
                                    <p>{format!("source_tag: {}", item.source_tag)}</p>
                                    <p class="preview-tag-line">
                                        <span class="tag-disk" style=tag_disk_style></span>
                                        <span>{format!("tags: {}", tags_label(&item.tags))}</span>
                                    </p>
                                    <p>{format!("pair {}: IB {:.3}", pair_index + 1, item.ib)}</p>
                                    <p>{format!(
                                        "frequency: {}",
                                        frequency
                                            .map(|v| format!("{v:.6}"))
                                            .unwrap_or_else(|| "inactive".to_string())
                                    )}</p>
                                    <p>{format!(
                                        "weight: {}",
                                        weight
                                            .map(|v| format!("{v:.6}"))
                                            .unwrap_or_else(|| "none".to_string())
                                    )}</p>
                                    <div class="plotly-hover-actions">
                                        <button on:click=move |_| on_jump.run(id)>"Jump To List"</button>
                                        <button on:click=move |_| on_select.run(id)>"Open Details"</button>
                                    </div>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()
            }}
            {move || {
                hovered_preview
                    .get()
                    .map(|(item, pair_index, frequency, weight)| {
                        let id = item.id;
                        let (left, top) = hover_card_position.get();
                        let tag_disk_style = tag_disk_style(&item.tags, &tag_color_map.get());
                        view! {
                            <div
                                class="plotly-hover-preview"
                                style=format!("left:{left}px; top:{top}px;")
                                on:mouseenter=move |_| {
                                    hover_generation.update(|generation| *generation = generation.wrapping_add(1));
                                    hover_card_hovered.set(true);
                                }
                                on:mouseleave=move |_| {
                                    hover_generation.update(|generation| *generation = generation.wrapping_add(1));
                                    hover_card_hovered.set(false);
                                    hover_id.set(None);
                                    hovered_pair.set(None);
                                }
                            >
                                <img src=item.image_data alt="hover preview" />
                                <div class="plotly-hover-meta">
                                    <p class="preview-source">{item.source}</p>
                                    <p>{format!("source_tag: {}", item.source_tag)}</p>
                                    <p class="preview-tag-line">
                                        <span class="tag-disk" style=tag_disk_style></span>
                                        <span>{format!("tags: {}", tags_label(&item.tags))}</span>
                                    </p>
                                    <p>{format!("pair {}: IB {:.3}", pair_index + 1, item.ib)}</p>
                                    <p>{format!(
                                        "frequency: {}",
                                        frequency
                                            .map(|v| format!("{v:.6}"))
                                            .unwrap_or_else(|| "inactive".to_string())
                                    )}</p>
                                    <p>{format!(
                                        "weight: {}",
                                        weight
                                            .map(|v| format!("{v:.6}"))
                                            .unwrap_or_else(|| "none".to_string())
                                    )}</p>
                                    <div class="plotly-hover-actions">
                                        <button on:click=move |_| on_jump.run(id)>"Jump To List"</button>
                                        <button on:click=move |_| on_select.run(id)>"Open Details"</button>
                                    </div>
                                </div>
                            </div>
                        }
                    })
            }}
        </section>
    }
}
