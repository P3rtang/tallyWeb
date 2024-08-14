use super::{CountableId, CountableStore};
use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMethod {
    Id(bool),
    Name(bool),
    Count(bool),
    Time(bool),
    CreatedAt(bool),
}

impl SortMethod {
    pub fn sort_by(
        &self,
    ) -> impl Fn(&CountableStore, &CountableId, &CountableId) -> std::cmp::Ordering {
        match self {
            Self::Id(false) => |_: &CountableStore, a: &CountableId, b: &CountableId| a.cmp(b),
            Self::Id(true) => {
                |_: &CountableStore, a: &CountableId, b: &CountableId| a.cmp(b).reverse()
            }
            Self::Name(false) => |store: &CountableStore, a: &CountableId, b: &CountableId| {
                store.name(a).cmp(&store.name(b))
            },
            Self::Name(true) => |store: &CountableStore, a: &CountableId, b: &CountableId| {
                store.name(a).cmp(&store.name(b)).reverse()
            },
            Self::Count(false) => |store: &CountableStore, a: &CountableId, b: &CountableId| {
                store.count(a).cmp(&store.count(b))
            },
            Self::Count(true) => |store: &CountableStore, a: &CountableId, b: &CountableId| {
                store.count(a).cmp(&store.count(b)).reverse()
            },
            Self::Time(false) => |store: &CountableStore, a: &CountableId, b: &CountableId| {
                store.time(a).cmp(&store.time(b))
            },
            Self::Time(true) => |store: &CountableStore, a: &CountableId, b: &CountableId| {
                store.time(a).cmp(&store.time(b)).reverse()
            },
            Self::CreatedAt(false) => |store: &CountableStore, a: &CountableId, b: &CountableId| {
                store.created_at(a).cmp(&store.created_at(b))
            },
            Self::CreatedAt(true) => |store: &CountableStore, a: &CountableId, b: &CountableId| {
                store.created_at(a).cmp(&store.created_at(b)).reverse()
            },
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Self::Id(b) => Self::Id(!b),
            Self::Name(b) => Self::Name(!b),
            Self::Count(b) => Self::Count(!b),
            Self::Time(b) => Self::Time(!b),
            Self::CreatedAt(b) => Self::CreatedAt(!b),
        }
    }

    pub fn is_reversed(&self) -> bool {
        match self {
            Self::Id(b) => *b,
            Self::Name(b) => *b,
            Self::Count(b) => *b,
            Self::Time(b) => *b,
            Self::CreatedAt(b) => *b,
        }
    }
}

impl Default for SortMethod {
    fn default() -> Self {
        Self::CreatedAt(false)
    }
}

impl From<String> for SortMethod {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Name" => Self::Name(false),
            "Count" => Self::Count(false),
            "Time" => Self::Time(false),
            "Id" => Self::Id(false),
            "CreatedAt" => Self::CreatedAt(false),
            _ => Default::default(),
        }
    }
}

impl From<SortMethod> for &str {
    fn from(val: SortMethod) -> Self {
        match val {
            SortMethod::Id(_) => "Id",
            SortMethod::Name(_) => "Name",
            SortMethod::Count(_) => "Count",
            SortMethod::Time(_) => "Time",
            SortMethod::CreatedAt(_) => "CreatedAt",
        }
    }
}

#[component]
pub fn SortSearch<S, K>(shown: S, search: RwSignal<String>, on_keydown: K) -> impl IntoView
where
    S: Fn() -> bool + 'static,
    K: Fn(ev::KeyboardEvent) + Copy + 'static,
{
    let sort_method = expect_context::<RwSignal<SortMethod>>();

    let select_sort = create_node_ref::<leptos::html::Select>();
    let on_sort = move |_| {
        sort_method.set(
            select_sort()
                .map(|nr| nr.value())
                .unwrap_or_default()
                .into(),
        )
    };

    create_isomorphic_effect(move |_| {
        select_sort().map(|rf| rf.set_value(sort_method.get_untracked().into()))
    });

    let reverse_order = move |_| sort_method.update(|s| *s = s.toggle());
    let arrow = move || {
        if sort_method().is_reversed() {
            "fa-solid fa-arrow-down"
        } else {
            "fa-solid fa-arrow-up"
        }
    };

    let is_searching = create_rw_signal(false);

    let search_input = create_node_ref::<leptos::html::Input>();

    let on_search = move |ev: ev::Event| {
        search.set(event_target_value(&ev));
    };

    let on_key = move |ev: ev::KeyboardEvent| {
        match ev.key().as_str() {
            "Escape" => {
                if let Some(i) = search_input() {
                    search.set(String::new());
                    i.set_value("");
                    let _ = i.blur();
                }
            }
            "Enter" => {
                if let Some(i) = search_input() {
                    let _ = i.blur();
                }
            }
            _ => {}
        }

        on_keydown(ev);
    };

    window_event_listener(ev::keydown, move |ev: ev::KeyboardEvent| {
        #[allow(clippy::single_match)]
        match ev.key().as_str() {
            "/" => is_searching.set(true),
            _ => {}
        }
    });

    create_effect(move |_| {
        is_searching();
        request_animation_frame(move || {
            search_input.get_untracked().map(|si| si.focus());
        })
    });

    view! {
        <Show when=shown fallback=|| ()>
            <div id="sort-search">
                <Show
                    when=is_searching
                    fallback=move || {
                        view! {
                            <button
                                id="search-button"
                                aria-label="search treeview"
                                on:click=move |_| {
                                    is_searching.set(true);
                                }
                            >

                                <i class="fa-solid fa-magnifying-glass"></i>
                            </button>
                        }
                    }
                >

                    <div id="search">
                        <input
                            id="search-input"
                            node_ref=search_input
                            on:keydown=on_key
                            on:input=on_search
                            on:focusout=move |_| {
                                if search_input().map(|si| si.value() == "").unwrap_or_default() {
                                    is_searching.set(false)
                                }
                            }
                        />

                    </div>
                </Show>
                <div id="sort">
                    <button aria-label="reverse treeview order" on:click=reverse_order>
                        <i class=arrow></i>
                    </button>
                    <select node_ref=select_sort on:change=on_sort>
                        <option value="Name">Name</option>
                        <option value="Count">Count</option>
                        <option value="Time">Time</option>
                        <option value="CreatedAt" selected>
                            Created At
                        </option>
                    </select>
                </div>
            </div>
        </Show>
    }
}
