use leptos::prelude::*;
use uuid::Uuid;

use crate::models::ImageRecord;

const WIDTH: f64 = 900.0;
const HEIGHT: f64 = 280.0;
const PAD_X: f64 = 46.0;
const PAD_Y: f64 = 24.0;

#[component]
pub fn ScatterPlot(
    images: Memo<Vec<ImageRecord>>,
    selected_id: RwSignal<Option<Uuid>>,
    hover_id: RwSignal<Option<Uuid>>,
    on_select: Callback<Uuid>,
    on_jump: Callback<Uuid>,
) -> impl IntoView {
    let extents = Memo::new(move |_| {
        let items = images.get();
        if items.is_empty() {
            return (0.0, 1.0, 0.0, 1.0);
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

        (
            min_ib,
            if min_ib == max_ib { min_ib + 1.0 } else { max_ib },
            min_f,
            if min_f == max_f { min_f + 1.0 } else { max_f },
        )
    });

    let project_x = move |ib: f64| {
        let (min_ib, max_ib, _, _) = extents.get();
        PAD_X + (ib - min_ib) / (max_ib - min_ib) * (WIDTH - PAD_X * 2.0)
    };

    let project_y = move |freq: f64| {
        let (_, _, min_f, max_f) = extents.get();
        HEIGHT - PAD_Y - (freq - min_f) / (max_f - min_f) * (HEIGHT - PAD_Y * 2.0)
    };

    let hovered = Memo::new(move |_| {
        hover_id
            .get()
            .and_then(|id| images.get().into_iter().find(|x| x.id == id))
    });

    view! {
        <section class="scatter-panel">
            <div class="scatter-header">
                <h2>"Scattering Plot (frequency vs IB)"</h2>
            </div>
            <div class="scatter-wrap">
                <svg viewBox=format!("0 0 {} {}", WIDTH, HEIGHT) class="scatter-svg">
                    <line x1=PAD_X y1=HEIGHT-PAD_Y x2=WIDTH-PAD_X y2=HEIGHT-PAD_Y class="axis" />
                    <line x1=PAD_X y1=PAD_Y x2=PAD_X y2=HEIGHT-PAD_Y class="axis" />
                    <text x=WIDTH/2.0 y=HEIGHT-4.0 class="axis-label">"IB"</text>
                    <text x=6 y=14 class="axis-label">"frequency"</text>

                    <For
                        each=move || images.get()
                        key=|item| item.id
                        children=move |item| {
                            let x = project_x(item.ib);
                            let y = project_y(item.frequency);
                            let id = item.id;
                            view! {
                                <circle
                                    cx=x
                                    cy=y
                                    r=5
                                    class=move || {
                                        if selected_id.get() == Some(id) { "dot selected" } else { "dot" }
                                    }
                                    on:mouseover=move |_| hover_id.set(Some(id))
                                    on:mouseleave=move |_| hover_id.set(None)
                                    on:click=move |_| on_select.run(id)
                                />
                            }
                        }
                    />
                </svg>

                {move || hovered.get().map(|item| {
                    let x = project_x(item.ib);
                    let y = project_y(item.frequency);
                    let id = item.id;
                    view! {
                        <div class="hover-card" style=format!("left:{}px; top:{}px;", x + 10.0, y + 10.0)>
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
        </section>
    }
}
