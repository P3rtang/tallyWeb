use leptos::*;
use web_sys::KeyboardEvent;

use crate::app::CounterList;

#[component]
pub fn SortSearch<F>(list: RwSignal<CounterList>, shown: F) -> impl IntoView
where
    F: Fn() -> bool + 'static,
{
    let (sort_method, set_sort_method) =
        create_slice(list, |list| list.sort.clone(), |list, new| list.sort = new);
    let select_sort = create_node_ref::<leptos::html::Select>();
    let on_sort = move |_| {
        set_sort_method.set(
            select_sort()
                .map(|nr| nr.value())
                .unwrap_or_default()
                .into(),
        )
    };

    create_effect(move |_| {
        select_sort().map(|rf| rf.set_value(sort_method.get_untracked().into()))
    });

    let reverse_order = move |_| set_sort_method.set(sort_method().toggle());
    let arrow = move || {
        if sort_method().is_reversed() {
            "fa-solid fa-arrow-down"
        } else {
            "fa-solid fa-arrow-up"
        }
    };

    let is_searching = create_rw_signal(false);

    let search_input = create_node_ref::<leptos::html::Input>();

    let on_search = move |ev: KeyboardEvent| {
        if ev.key() == "Escape" || ev.key() == "Enter" {
            let _ = search_input().unwrap().blur();
        }
        list.update(|l| l.search(&search_input().unwrap().value()));
        ev.stop_propagation();
    };

    view! {
        <Show
            when=shown
            fallback=|| ()
        >
        <div id="sort-search">
            <Show
                when=is_searching
                fallback=move || view! {
                    <button
                        id="search-button"
                        on:click=move |_| {
                            is_searching.set(true);
                        }
                    >
                        <i class="fa-solid fa-magnifying-glass"></i>
                    </button>
                }
            >
                <div id="search">
                    <input
                        id="search-input"
                        node_ref=search_input
                        on:keyup=on_search
                        on:focusout=move |_| {
                            if search_input().map(|si| si.value() == "").unwrap_or_default() {
                                is_searching.set(false)
                            }
                        }
                    />
                </div>
            </Show>
            <div id="sort">
                <button on:click=reverse_order>
                    <i class=arrow></i>
                </button>
                <select node_ref=select_sort on:change=on_sort>
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
