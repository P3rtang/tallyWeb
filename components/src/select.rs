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

#[derive(Clone, Default)]
#[slot]
pub struct SelectButton {
    #[prop(into, optional)]
    attrs: AttributeFn,
}

#[component]
pub fn Select<IV, V, T>(
    #[prop(into)] options: Signal<Vec<T>>,
    #[prop(into, optional)] selected: Option<Signal<T>>,
    view: V,

    #[prop(optional)] select_input: SelectInput,
    #[prop(optional)] select_button: SelectButton,
) -> impl IntoView
where
    T: ToString + Clone + Send + Sync + 'static,
    V: Fn(String) -> IV + Clone + Send + Sync + 'static,
    IV: IntoView + Clone + Send + Sync + 'static,
{
    let hidden_select_ref = NodeRef::<leptos::html::Input>::new();
    let show_custom = RwSignal::new(false);
    let selection = RwSignal::new(selected.map(|sel| sel.get_untracked().to_string()));
    let options = StoredValue::new(
        options
            .get()
            .into_iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>(),
    );
    let view = StoredValue::new(view);
    let select_button = StoredValue::new(select_button);

    let options_view = options
        .get_value()
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

    Effect::new(move |_| {
        show_custom.set(true);
        if let Some(node) = hidden_select_ref.get() {
            selection.set(Some(
                options
                    .get_value()
                    .into_iter()
                    .find_map(|o| (o == node.value()).then_some(o))
                    .unwrap_or_default(),
            ));
        }
    });

    Effect::new(move |_| {
        if let Some(node) = hidden_select_ref.get() {
            node.set_value(&selection().unwrap_or_default())
        }
    });

    view! {
        <Show
            when=show_custom
            fallback=move || {
                view! { <select>{options_view.clone()}</select> }
            }
        >
            <input
                {..select_input.attrs.call()}
                prop:value=selection
                type="hidden"
                node_ref=hidden_select_ref
            />
            <SelectOver
                options=options.get_value()
                selection
                view=view.get_value()
                select_button=select_button.get_value()
            />
        </Show>
    }
}

#[component]
pub fn SelectOver<V, IV>(
    #[prop(into)] options: Signal<Vec<String>>,
    selection: RwSignal<Option<String>>,
    view: V,

    select_button: SelectButton,
) -> impl IntoView
where
    V: Fn(String) -> IV + Send + Sync + 'static,
    IV: IntoView + Send + Sync + 'static,
{
    let options = StoredValue::new(
        options
            .get()
            .into_iter()
            .map(|o| o.to_string())
            .collect::<Vec<_>>(),
    );
    let show_options = RwSignal::new(false);

    let toggle_show = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        show_options.update(|s| *s = !*s)
    };

    let on_option = move |val| {
        selection.set(Some(val));
        show_options.set(false);
    };

    let toggle_style = move || if show_options() { "rotate(180deg)" } else { "" };

    let options_list_ref = NodeRef::<leptos::html::Div>::new();

    let max_height = RwSignal::new(String::new());

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
            let mut mut_options = options.get_value();
            mut_options.sort_by(sorter.sort());
            mut_options
        } else {
            options.get_value()
        }
    });

    let selected_bg = move |idx: usize, option: String| {
        if key_input().is_some() && idx == 0
            || key_input().is_none() && Some(option) == selection.get()
        {
            "var(--accent, #3584E4)"
        } else {
            ""
        }
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
                selection.set(Some(options_memo.get_untracked()[0].clone()));
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

    view! {
        <style>
            r#"
            select-options {
                scrollbar-width: thin;
                scrollbar-color: rgba(0, 0, 0, 0.32) transparent;
            
                &>div {
                    position: relative;
                }
            }
            "#
        </style>
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
                                view! { <span>{selection().unwrap_or_default()}</span> }
                            }
                        >
                            {get_label}
                        </Show>
                    </label>
                    <button
                        type="button"
                        id="dropdown-button"
                        {..select_button.attrs.call()}
                        on:click=toggle_show
                    >
                        <div>
                            <img
                                src="/icons/dropdown.svg"
                                width="24px"
                                height="24px"
                                style:transform=toggle_style
                            />
                        </div>
                    </button>
                </select-view>
                <Show when=show_options>
                    <select-options style:display="block" style:max-height=max_height>

                        {options_memo()
                            .into_iter()
                            .enumerate()
                            .map(move |(idx, option)| {
                                let option = StoredValue::new(option.to_string());
                                view! {
                                    <select-option
                                        on:click=move |_| on_option(option.get_value())
                                        style:display="block"
                                        style:background=move || selected_bg(
                                            idx,
                                            option.get_value(),
                                        )
                                    >
                                        {option.get_value()}
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
