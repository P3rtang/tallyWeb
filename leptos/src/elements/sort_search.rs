use leptos::*;

use crate::countable::SortCountable;

#[component]
pub fn SortSearch<F>(sort_method: RwSignal<SortCountable>, shown: F) -> impl IntoView
where
    F: Fn() -> bool + 'static,
{
    let select_sort = create_node_ref::<leptos::html::Select>();
    let on_change = move |_| {
        sort_method.set(
            select_sort()
                .map(|nr| nr.value())
                .unwrap_or_default()
                .into(),
        )
    };

    let reverse_order = move |_| sort_method.update(|s| *s = s.toggle());
    let arrow = move || {
        if sort_method().is_reversed() {
            "fa-solid fa-arrow-down"
        } else {
            "fa-solid fa-arrow-up"
        }
    };

    view! {
        <Show
            when=shown
            fallback=|| ()
        >
        <div id="sort-search">
            <button id="search">
                <i class="fa-solid fa-magnifying-glass"></i>
            </button>
            <div id="sort">
                <button on:click=reverse_order>
                    <i class=arrow></i>
                </button>
                <select node_ref=select_sort on:change=on_change>
                    <option value="Name">Name</option>
                    <option value="Count">Count</option>
                    <option value="Time">Time</option>
                    <option value="CreatedAt">Created At</option>
                </select>
            </div>
        </div>
        </Show>
    }
}
