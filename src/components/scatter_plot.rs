use leptos::prelude::*;
use uuid::Uuid;

use crate::models::ImageRecord;

const WIDTH: f64 = 900.0;
const HEIGHT: f64 = 280.0;
const PAD_X: f64 = 46.0;
const PAD_Y: f64 = 24.0;

fn parse_f64_opt(raw: &str) -> Option<f64> {
    raw.trim().parse::<f64>().ok()
}

#[component]
pub fn ScatterPlot(
    images: Memo<Vec<ImageRecord>>,
    selected_id: RwSignal<Option<Uuid>>,
    hover_id: RwSignal<Option<Uuid>>,
    on_select: Callback<Uuid>,
    on_jump: Callback<Uuid>,
) -> impl IntoView {
    let show_axis_menu = RwSignal::new(false);
    let x_min_input = RwSignal::new(String::new());
    let x_max_input = RwSignal::new(String::new());
    let y_min_input = RwSignal::new(String::new());
    let y_max_input = RwSignal::new(String::new());
    let manual_limits = RwSignal::new(None::<(f64, f64, f64, f64)>);
    let axis_error = RwSignal::new(String::new());

    let auto_extents = Memo::new(move |_| {
        let items = images.get();
        if items.is_empty() {
            return (0.0, 1.0, 0.0, 1.0, 0.0, 1.0);
        }
        let min_ib = items.iter().map(|x| x.ib).fold(f64::INFINITY, f64::min);
        let max_ib = items.iter().map(|x| x.ib).fold(f64::NEG_INFINITY, f64::max);
        let min_f = items
            .iter()
            .map(|x| x.frequency)
            .fold(f64::INFINITY, f64::min);
        let max_f = items
            .iter()
            .map(|x| x.frequency)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_w = items
            .iter()
            .map(|x| x.weight)
            .fold(f64::INFINITY, f64::min);
        let max_w = items
            .iter()
            .map(|x| x.weight)
            .fold(f64::NEG_INFINITY, f64::max);

        (
            min_ib,
            if min_ib == max_ib { min_ib + 1.0 } else { max_ib },
            min_f,
            if min_f == max_f { min_f + 1.0 } else { max_f },
            min_w,
            if min_w == max_w { min_w + 1.0 } else { max_w },
        )
    });

    let extents = Memo::new(move |_| {
        let (auto_x_min, auto_x_max, auto_y_min, auto_y_max, min_w, max_w) = auto_extents.get();
        if let Some((x_min, x_max, y_min, y_max)) = manual_limits.get() {
            (x_min, x_max, y_min, y_max, min_w, max_w)
        } else {
            (auto_x_min, auto_x_max, auto_y_min, auto_y_max, min_w, max_w)
        }
    });

    let project_x = move |ib: f64| {
        let (min_ib, max_ib, _, _, _, _) = extents.get();
        PAD_X + (ib - min_ib) / (max_ib - min_ib) * (WIDTH - PAD_X * 2.0)
    };

    let project_y = move |freq: f64| {
        let (_, _, min_f, max_f, _, _) = extents.get();
        HEIGHT - PAD_Y - (freq - min_f) / (max_f - min_f) * (HEIGHT - PAD_Y * 2.0)
    };

    let project_opacity = move |weight: f64| {
        let (_, _, _, _, min_w, max_w) = extents.get();
        let normalized = ((weight - min_w) / (max_w - min_w)).clamp(0.0, 1.0);
        0.2 + normalized * 0.8
    };

    let hovered = Memo::new(move |_| {
        hover_id
            .get()
            .and_then(|id| images.get().into_iter().find(|x| x.id == id))
    });

    let apply_axis_limits = move |_| {
        let (auto_x_min, auto_x_max, auto_y_min, auto_y_max, _, _) = auto_extents.get();

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
                <button on:click=move |_| show_axis_menu.update(|v| *v = !*v)>"Axis Limits"</button>
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
                <div
                    class="scatter-hit-area"
                    on:mouseleave=move |_| hover_id.set(None)
                >
                <svg viewBox=format!("0 0 {} {}", WIDTH, HEIGHT) class="scatter-svg">
                    <line x1=PAD_X y1=HEIGHT-PAD_Y x2=WIDTH-PAD_X y2=HEIGHT-PAD_Y class="axis" />
                    <line x1=PAD_X y1=PAD_Y x2=PAD_X y2=HEIGHT-PAD_Y class="axis" />
                    <text x=WIDTH/2.0 y=HEIGHT-4.0 class="axis-label">"IB"</text>
                    <text x=6 y=14 class="axis-label">"frequency"</text>
                    {move || {
                        let (min_x, max_x, _, _, _, _) = extents.get();
                        (0..=4)
                            .map(|i| {
                                let t = i as f64 / 4.0;
                                let x = PAD_X + t * (WIDTH - PAD_X * 2.0);
                                let value = min_x + t * (max_x - min_x);
                                view! {
                                    <>
                                        <line
                                            x1=x
                                            y1=HEIGHT-PAD_Y
                                            x2=x
                                            y2=HEIGHT-PAD_Y+4.0
                                            class="tick-line"
                                        />
                                        <text x=x y=HEIGHT-PAD_Y+16.0 class="tick-label">{format!("{value:.2}")}</text>
                                    </>
                                }
                            })
                            .collect_view()
                    }}
                    {move || {
                        let (_, _, min_y, max_y, _, _) = extents.get();
                        (0..=4)
                            .map(|i| {
                                let t = i as f64 / 4.0;
                                let y = HEIGHT - PAD_Y - t * (HEIGHT - PAD_Y * 2.0);
                                let value = min_y + t * (max_y - min_y);
                                view! {
                                    <>
                                        <line
                                            x1=PAD_X-4.0
                                            y1=y
                                            x2=PAD_X
                                            y2=y
                                            class="tick-line"
                                        />
                                        <text x=PAD_X-8.0 y=y+3.0 class="tick-label y-tick">{format!("{value:.2}")}</text>
                                    </>
                                }
                            })
                            .collect_view()
                    }}

                    <For
                        each=move || images.get()
                        key=|item| item.id
                        children=move |item| {
                            let ib = item.ib;
                            let freq = item.frequency;
                            let weight = item.weight;
                            let id = item.id;
                            view! {
                                <circle
                                    cx=move || project_x(ib)
                                    cy=move || project_y(freq)
                                    r=5
                                    style=move || format!("opacity: {:.3};", project_opacity(weight))
                                    class=move || {
                                        if selected_id.get() == Some(id) { "dot selected" } else { "dot" }
                                    }
                                    on:mouseover=move |_| hover_id.set(Some(id))
                                    on:click=move |_| on_select.run(id)
                                />
                            }
                        }
                    />
                </svg>

                {move || hovered.get().map(|item| {
                    let ib = item.ib;
                    let freq = item.frequency;
                    let id = item.id;
                    view! {
                        <div
                            class="hover-card"
                            style=move || {
                                format!(
                                    "left:{}px; top:{}px;",
                                    project_x(ib) + 10.0,
                                    project_y(freq) + 10.0
                                )
                            }
                        >
                            <img src=item.image_data alt="hover preview" />
                            <p>{item.source.clone()}</p>
                            <p>{format!("IB: {:.3}, freq: {:.3}", item.ib, item.frequency)}</p>
                            <div class="hover-actions">
                                <button on:click=move |_| on_jump.run(id)>"Jump To List"</button>
                                <button on:click=move |_| on_select.run(id)>"Open Details"</button>
                            </div>
                        </div>
                    }
                })}
                </div>
            </div>
        </section>
    }
}
