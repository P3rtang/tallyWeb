use super::CloseOverlays;
use fuzzy_sort::*;
use leptos::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectOption {
    name: String,
    value: String,
}

impl Sortable for SelectOption {
    fn as_str(&self) -> &str {
        &self.name
    }
}

impl From<(String, String)> for SelectOption {
    fn from(value: (String, String)) -> Self {
        Self {
            name: value.0,
            value: value.1,
        }
    }
}

impl From<(&str, &str)> for SelectOption {
    fn from(value: (&str, &str)) -> Self {
        Self {
            name: value.0.to_string(),
            value: value.1.to_string(),
        }
    }
}

#[component]
pub fn Select(
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    #[prop(into)] options: Vec<SelectOption>,
    #[prop(into)] selected: MaybeSignal<SelectOption>,
) -> impl IntoView {
    let attrs = store_value(attrs);
    let hidden_select_ref = create_node_ref::<html::Input>();
    let show_custom = create_rw_signal(false);
    let selection = create_rw_signal(SelectOption::default());
    let options = store_value(options);

    create_isomorphic_effect(move |_| {
        selection.set(selected.get());
    });

    let options_view = options()
        .into_iter()
        .map(move |option| {
            view! {
                <option
                    value=option.value.clone()
                    selected=move || selection().value == option.value
                >
                    {option.name}
                </option>
            }
        })
        .collect_view();

    create_effect(move |_| {
        show_custom.set(true);
        if let Some(node) = hidden_select_ref.get() {
            selection.set(
                options()
                    .into_iter()
                    .find_map(|o| (o.value == node.value()).then_some(o))
                    .unwrap_or_default(),
            );
        }
    });

    create_effect(move |_| {
        if let Some(node) = hidden_select_ref.get() {
            node.set_value(&selection().value)
        }
    });

    view! {
        <Show
            when=show_custom
            fallback=move || {
                view! { <select {..attrs()}>{options_view.clone()}</select> }
            }
        >

            <SelectOver options=options() selection/>
            <input {..attrs()} type="hidden" node_ref=hidden_select_ref/>
        </Show>
    }
}

#[component]
pub fn SelectOver(
    #[prop(into)] options: Vec<SelectOption>,
    selection: RwSignal<SelectOption>,
) -> impl IntoView {
    let options = store_value(options);
    let show_options = create_rw_signal(false);

    let toggle_show = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        show_options.update(|s| *s = !*s)
    };

    let on_option = move |val| {
        selection.set(val);
        show_options.set(false);
    };

    let toggle_style = move || if show_options() { "rotate(180deg)" } else { "" };

    let options_list_ref = create_node_ref::<html::Div>();

    let max_height = create_rw_signal(None::<String>);

    create_effect(move |_| {
        if let Some(node) = options_list_ref() {
            request_animation_frame(move || {
                let y = node.get_bounding_client_rect().top();
                let screen_height = window()
                    .inner_height()
                    .ok()
                    .and_then(|js_val| js_val.as_f64())
                    .unwrap_or(1080.0);
                max_height.set(Some(format!("{}px", screen_height - y)))
            })
        }
    });

    let key_input = create_rw_signal(None::<String>);
    let options_memo = create_memo(move |_| {
        if let Some(i) = key_input() {
            let sorter = SimpleMatch::new(i);
            let mut mut_options = options();
            mut_options.sort_by(sorter.sort());
            mut_options
        } else {
            options()
        }
    });

    let selected_bg = move |idx: usize, option: SelectOption| {
        if key_input().is_some() && idx == 0
            || key_input().is_none() && option.value == selection().value
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
                selection.set(options_memo.get_untracked()[0].clone());
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

    if let Some(close_signal) = use_context::<RwSignal<CloseOverlays>>() {
        create_effect(move |_| {
            close_signal.track();
            show_options.set(false);
        });
    } else {
        logging::warn!("No `close overlay` signal available");
    }

    on_cleanup(|| key_listener.remove());

    let get_label = move || key_input().unwrap_or(selection().name);

    view! {
        <style>
            r#"select-options {
                scrollbar-width: thin;
                scrollbar-color: rgba(0, 0, 0, 0.32) transparent;
            }"#
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
                            fallback=move || view! { <span>{selection().name}</span> }
                        >
                            {get_label}
                        </Show>
                    </label>
                    <button
                        type="button"
                        id="dropdown-button"
                        on:click=toggle_show
                        style:height="40px"
                        style:width="40px"
                    >
                        <img
                            src="/icons/dropdown.svg"
                            width="24px"
                            height="24px"
                            style:transform=toggle_style
                        />
                    </button>
                </select-view>
                <Show when=show_options>
                    <select-options style:display="block" style:max-height=max_height>

                        {options_memo()
                            .into_iter()
                            .enumerate()
                            .map(move |(idx, option)| {
                                let option = store_value(option);
                                view! {
                                    <select-option
                                        on:click=move |_| on_option(option())
                                        style:display="block"
                                        style:background=move || selected_bg(idx, option())
                                    >
                                        {option().name}
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
