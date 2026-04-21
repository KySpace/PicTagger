use leptos::prelude::*;

#[component]
pub fn IbFilterBar(
    filter_ib_min: RwSignal<Option<f64>>,
    filter_ib_max: RwSignal<Option<f64>>,
) -> impl IntoView {
    let min_value = move || filter_ib_min.get().map(|v| v.to_string()).unwrap_or_default();
    let max_value = move || filter_ib_max.get().map(|v| v.to_string()).unwrap_or_default();

    view! {
        <section class="filter-bar">
            <h2>"IB Filter"</h2>
            <div class="filter-inputs">
                <label>
                    "Min IB"
                    <input
                        type="number"
                        step="any"
                        prop:value=min_value
                        on:input=move |ev| {
                            let raw = event_target_value(&ev);
                            if raw.trim().is_empty() {
                                filter_ib_min.set(None);
                            } else if let Ok(v) = raw.parse::<f64>() {
                                filter_ib_min.set(Some(v));
                            }
                        }
                    />
                </label>
                <label>
                    "Max IB"
                    <input
                        type="number"
                        step="any"
                        prop:value=max_value
                        on:input=move |ev| {
                            let raw = event_target_value(&ev);
                            if raw.trim().is_empty() {
                                filter_ib_max.set(None);
                            } else if let Ok(v) = raw.parse::<f64>() {
                                filter_ib_max.set(Some(v));
                            }
                        }
                    />
                </label>
                <button on:click=move |_| {
                    filter_ib_min.set(None);
                    filter_ib_max.set(None);
                }>
                    "Reset"
                </button>
            </div>
        </section>
    }
}
