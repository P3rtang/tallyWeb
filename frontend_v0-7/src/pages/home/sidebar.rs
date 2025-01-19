use super::*;

#[component]
pub(crate) fn SidebarContent(#[prop(into)] width: Signal<usize>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let selection = expect_context::<Memo<Selection>>();

    let width = move || format!("{}px", width.get());

    let is_selected = move |key: CountableId| selection.get().contains(&key);

    let action = ServerAction::<api::CreateCountable>::new();

    Effect::new(move |_| {
        match action.value().get() {
            Some(Ok(countables)) => {
                store.update(|s| countables.into_iter().for_each(|c| s.insert(c)))
            }
            // TODO: add in logging of server error with messagejar
            Some(Err(_err)) => (),
            None => (),
        }
    });

    let row_children = move |countable| view! { <TreeRow countable /> }.into_any();

    let input_ref = NodeRef::<html::Input>::new();
    let (show_search, set_show_search) = signal(false);
    let (search, set_search) = signal(None);

    let height = move || if show_search.get() { "100px" } else { "0px" };

    let handle_change = move |ev| {
        let value = event_target_value(&ev);

        if value.is_empty() {
            set_search.set(None)
        } else {
            set_search(Some(value))
        }
    };

    let handle_search = move |_| set_show_search.set(!show_search.get_untracked());

    let on_focus_out = move |_| {
        if search.get().is_none() {
            set_show_search.set(false)
        }
    };

    Effect::new(move |_| {
        if let Some(input_ref) = input_ref.get_untracked() {
            if show_search.get() {
                request_animation_frame(move || _ = input_ref.focus());
            }
        }
    });

    let each = move || {
        let mut root = store
            .get()
            .filter(|c| {
                c.name()
                    .to_lowercase()
                    .contains(&search.get().unwrap_or_default().to_lowercase())
            })
            .root_node_ids()
            .into_iter()
            .collect::<Vec<_>>();
        root.sort_by_key(|a| store.get().name(a));
        root
    };

    let children = move |c| {
        let mut children = store.get().children(&c);
        children.sort_by_key(|c| store.get_untracked().name(c));
        children
    };

    view! {
        <div class=style::sidebar>
            <Navbar on_search=handle_search />
            <div class=style::search_box style:max-height=height >
                <div>
                    <TextField
                        input_ref
                        id="search-filter"
                        on:blur=on_focus_out
                        on:input=handle_change
                    />
                </div>
            </div>
            <div style:width=width>
                <List
                    each
                    key=|c| *c
                    children
                >
                    <RowSlot is_selected children=row_children slot/>
                    <Separator slot><hr /></Separator>
                </List>
                <ActionForm action style:padding="0px 16px">
                    <input type="hidden" name="kind" value=CountableKind::Counter.to_string() />
                    <Button class:hover-darken=true style:width="100%" attr:r#type="submit">
                        <div>New Counter</div>
                    </Button>
                </ActionForm>
            </div>
        </div>
    }
}

#[component]
fn TreeRow(countable: CountableId) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let session = expect_context::<RwSignal<UserSession>>();
    let selection = expect_context::<Memo<Selection>>();
    let has_children = move || matches!(store.get().kind(&countable), CountableKind::Counter);

    let include_selection = move || {
        let mut sel = selection.get();
        if !sel.slct.remove(&countable) {
            sel.slct.insert(countable);
        }
        sel
    };

    let href = move || {
        format!(
            "?{}",
            serde_qs::to_string(&include_selection()).unwrap_or_default()
        )
    };

    let name = Signal::derive(move || store.get().name(&countable));

    let action = ServerAction::<api::CreateCountable>::new();

    Effect::new(move |_| match action.value().get() {
        Some(Ok(phase)) => store.update(|s| phase.into_iter().for_each(|p| s.insert(p))),
        Some(Err(_err)) => (),
        None => (),
    });

    view! {
        <A href style:width="100%">{name}</A>
        <Show when=has_children>
            <ActionForm action style:margin-right="2px">
                <session::SessionFormInput session />
                <input type="hidden" name="kind" value="Phase" />
                <input type="hidden" name="parent" value=countable.0.to_string() />
                <Button
                    size=ButtonSize::Small
                    rounding=ButtonRounding::Full
                    hover=ButtonHover::Darken
                    attr:r#type="submit"
                >
                    <div class=style::add_phase>+</div>
                </Button>
            </ActionForm>
        </Show>
    }
}

#[component]
fn Navbar(#[prop(into)] on_search: EventCallback<ev::MouseEvent>) -> impl IntoView {
    view! {
        <nav class=style::sidebar_navbar>
            <Button
                hover=ButtonHover::Lighten
                style:background="transparent"
                on:mousedown=move |ev| on_search.call(ev)
            >
                <Icon kind=IconKind::Search />
            </Button>
        </nav>
    }
}
