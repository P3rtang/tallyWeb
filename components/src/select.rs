use super::*;
use fuzzy_sort::*;
use leptos::{
    attr::{
        any_attribute::{AnyAttribute, IntoAnyAttribute},
        Attribute,
    },
    ev,
    prelude::*,
};

#[derive(Clone, Default)]
#[slot]
pub struct SelectInput {
    #[prop(into, optional)]
    attrs: AttributeFn,
}

#[derive(Clone)]
#[slot]
pub struct SelectButton<T>
where
    T: ToString + Clone + Send + Sync + 'static,
{
    #[prop(into, optional)]
    attrs: AttributeFn,

    #[prop(into, optional)]
    children: SelectButtonChild<T>,
}

impl<T> Default for SelectButton<T>
where
    T: ToString + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            attrs: AttributeFn::default(),
            children: SelectButtonChild::<T>::default(),
        }
    }
}

#[derive(Clone)]
pub struct SelectButtonChild<T>(Arc<dyn Fn(SelectState<T>) -> AnyView + Send + Sync + 'static>)
where
    T: ToString + Clone + Send + Sync + 'static;

impl<T> Default for SelectButtonChild<T>
where
    T: ToString + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self(Arc::new(move |state| {
            let toggle_style = move || {
                if state.is_expanded {
                    "rotate(180deg)"
                } else {
                    ""
                }
            };

            view! {
                <div style:font-size="48px" style:line-height="0px" style:transform=toggle_style>
                    "⌄"
                </div>
            }
            .into_any()
        }))
    }
}

impl<T, F, IV> From<F> for SelectButtonChild<T>
where
    T: ToString + Clone + Send + Sync + 'static,
    F: Fn(SelectState<T>) -> IV + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(move |state| value(state).into_any()))
    }
}

#[derive(Clone)]
pub struct SelectState<T>
where
    T: ToString + Clone + Send + Sync + 'static,
{
    pub is_expanded: bool,
    pub selection: Option<T>,
}

#[component]
pub fn Select<IV, V, T>(
    #[prop(into)] options: Signal<Vec<T>>,
    #[prop(into, optional)] value: Option<Signal<T>>,
    #[prop(into, optional)] on_change: EventCallback<Option<T>>,

    view: V,

    #[prop(optional)] select_input: SelectInput,
    #[prop(optional)] select_button: SelectButton<T>,
) -> impl IntoView
where
    T: Sortable + ToString + PartialEq + Clone + Send + Sync + 'static,
    V: Fn(T) -> IV + Clone + Send + Sync + 'static,
    IV: IntoView + Clone + Send + Sync + 'static,
{
    let hidden_select_ref = NodeRef::<leptos::html::Input>::new();
    let show_custom = RwSignal::new(false);
    let view = StoredValue::new(view);
    let select_button = StoredValue::new(select_button);

    let default_value = RwSignal::new(None::<T>);

    let selection = Memo::new(move |_| {
        if value.is_some() {
            value.get()
        } else {
            default_value.get()
        }
    });

    let handle_change = StoredValue::new(move |t: Option<T>| {
        default_value.set(t.clone());
        on_change.call(t);
    });

    let options_view = options
        .get()
        .into_iter()
        .map(move |option| {
            let option = StoredValue::new(option);

            view! {
                <option
                    value=option.get_value().to_string()
                    selected=move || Some(option.get_value()) == selection.get()
                >
                    {view.get_value()(option.get_value())}
                </option>
            }
        })
        .collect_view();

    Effect::new(move |_| show_custom.set(true));

    Effect::new(move |_| {
        if let Some(node) = hidden_select_ref.get() {
            node.set_value(&selection.get().map(|s| s.to_string()).unwrap_or_default())
        }
    });

    view! {
        <select
            style:display=move || if show_custom() { "none" } else { "block" }
            disabled=show_custom
        >
            {options_view.clone()}
        </select>
        <input
            {..select_input.attrs.call()}
            prop:value=move || selection.get().map(|s| s.to_string()).unwrap_or_default()
            type="hidden"
        />
        <SelectOver
            style:display=move || if show_custom() { "block" } else { "none" }
            options
            selection
            on_change=handle_change.get_value()
            view=view.get_value()
            select_button=select_button.get_value()
        />
    }
}

#[component]
pub fn SelectOver<V, IV, T>(
    #[prop(into)] options: Signal<Vec<T>>,
    #[prop(into)] selection: Signal<Option<T>>,
    #[prop(into)] on_change: EventCallback<Option<T>>,
    view: V,

    select_button: SelectButton<T>,
) -> impl IntoView
where
    T: Sortable + PartialEq + ToString + Clone + Send + Sync + 'static,
    V: Fn(T) -> IV + Send + Sync + 'static,
    IV: IntoView + Send + Sync + 'static,
{
    let handle_change = StoredValue::new(on_change);
    let show_options = RwSignal::new(false);

    let toggle_show = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        show_options.update(|s| *s = !*s)
    };

    let on_option = StoredValue::new(move |val| {
        handle_change.get_value().call(Some(val));
        show_options.set(false);
    });

    let options_list_ref = NodeRef::<leptos::html::Div>::new();

    let max_height = RwSignal::new(String::new());

    // TODO: recalculate this on opening the options
    // TODO: create another view for mobile
    Effect::new(move |_| {
        if let Some(node) = options_list_ref.get() {
            request_animation_frame(move || {
                let y = node.get_bounding_client_rect().top();
                let screen_height = window()
                    .inner_height()
                    .ok()
                    .and_then(|js_val| js_val.as_f64())
                    .unwrap_or(1080.0);
                max_height.set(format!("{}px", screen_height - y))
            })
        }
    });

    let key_input = RwSignal::new(None::<String>);
    let options_memo = Memo::new(move |_| {
        if let Some(i) = key_input() {
            let sorter = SimpleMatch::new(i);
            let mut mut_options = options.get();
            mut_options.sort_by(sorter.sort());
            mut_options
        } else {
            options.get()
        }
    });

    let selected_bg = move |idx: usize, option: T| {
        if key_input().is_some() && idx == 0
            || key_input().is_none() && Some(option) == selection.get()
        {
            "var(--accent, #3584E4)"
        } else {
            ""
        }
    };

    let is_selected = move |idx, option| {
        key_input().is_some() && idx == 0
            || key_input().is_none() && Some(option) == selection.get()
    };

    let key_listener = window_event_listener(ev::keydown, move |ev| {
        if !show_options() {
            return;
        }

        match ev.key().as_str() {
            "Backspace" => key_input.set({
                if key_input().is_some_and(|i| i.len() > 1) {
                    let i = key_input().unwrap();
                    Some(i[0..i.len() - 1].to_string())
                } else {
                    None
                }
            }),
            " " if key_input().is_none() => {}
            "Enter" if key_input().is_some() => {
                handle_change
                    .get_value()
                    .call(Some(options_memo.get_untracked()[0].clone()));
                key_input.set(None);
            }
            "Escape" => {
                key_input.set(None);
                show_options.set(false);
            }
            k if k.len() == 1 => {
                ev.stop_propagation();
                ev.prevent_default();
                key_input.set(Some(key_input().unwrap_or_default() + k))
            }
            _ => {}
        }
    });

    // TODO: readd a close signal

    on_cleanup(|| key_listener.remove());

    let get_label =
        move || key_input().unwrap_or(selection().map(|s| s.to_string()).unwrap_or_default());

    let select_state = Signal::derive(move || SelectState {
        is_expanded: show_options.get(),
        selection: selection.get(),
    });

    view! {
        // TODO: add a page body click event listener to the page context API
        <custom-select>
            <div node_ref=options_list_ref>
                <select-view style:display="flex">
                    <label
                        style:align-content="center"
                        style:width="100%"
                        on:click=|ev| ev.stop_propagation()
                        for="dropdown-button"
                    >
                        <Show
                            when=move || key_input().is_some()
                            fallback=move || {
                                view! {
                                    <span>
                                        {selection().map(|s| s.to_string()).unwrap_or_default()}
                                    </span>
                                }
                            }
                        >
                            {get_label}
                        </Show>
                    </label>
                    <button
                        type="button"
                        id="dropdown-button"
                        class="hover-darken icon"
                        on:click=toggle_show
                        aria_label="show options"
                    >
                        <div>{move || (select_button.children.0)(select_state.get())}</div>
                    </button>
                </select-view>
                <Show when=show_options>
                    <select-options style:display="block" style:max-height=max_height>
                        {options_memo()
                            .into_iter()
                            .enumerate()
                            .map(move |(idx, option)| {
                                let option = StoredValue::new(option);
                                view! {
                                    <select-option
                                        on:click=move |_| on_option.get_value()(option.get_value())
                                        style:display="block"
                                        selected=is_selected(idx, option.get_value())
                                    >
                                        {option.get_value().to_string()}
                                    </select-option>
                                }
                            })
                            .collect_view()}

                    </select-options>
                </Show>
            </div>
        </custom-select>
    }
}
