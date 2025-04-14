use super::*;

type SortFn = Box<dyn FnMut(&CountableId, &CountableId) -> std::cmp::Ordering>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sort {
    Name { reverse: bool },
    Count { reverse: bool },
    Time { reverse: bool },
    CreatedOn { reverse: bool },
}

impl Sort {
    fn sort_fn(self, store: RwSignal<CountableStore>) -> SortFn {
        let (mut func, rev): (SortFn, bool) = match self {
            Sort::Name { reverse } => (
                Box::new(move |a, b| store.get().name(b).cmp(&store.get().name(a))),
                reverse,
            ),
            Sort::Count { reverse } => (
                Box::new(move |a, b| {
                    store
                        .get()
                        .recursive_ref()
                        .count(a)
                        .cmp(&store.get().recursive_ref().count(b))
                }),
                reverse,
            ),
            Sort::Time { reverse } => (
                Box::new(move |a, b| {
                    store
                        .get()
                        .recursive_ref()
                        .time(a)
                        .cmp(&store.get().recursive_ref().time(b))
                }),
                reverse,
            ),
            Sort::CreatedOn { reverse } => (
                Box::new(move |a, b| store.get().created_at(a).cmp(&store.get().created_at(b))),
                reverse,
            ),
        };

        if rev {
            Box::new(move |a, b| func(a, b).reverse())
        } else {
            func
        }
    }

    fn is_reversed(&self) -> bool {
        match self {
            Sort::Name { reverse } => *reverse,
            Sort::Count { reverse } => *reverse,
            Sort::Time { reverse } => *reverse,
            Sort::CreatedOn { reverse } => *reverse,
        }
    }

    fn reverse(&mut self) {
        match self {
            Sort::Name { reverse } => *reverse = !*reverse,
            Sort::Count { reverse } => *reverse = !*reverse,
            Sort::Time { reverse } => *reverse = !*reverse,
            Sort::CreatedOn { reverse } => *reverse = !*reverse,
        }
    }

    fn set_reverse(&mut self, rev: bool) {
        match self {
            Sort::Name { reverse } => *reverse = rev,
            Sort::Count { reverse } => *reverse = rev,
            Sort::Time { reverse } => *reverse = rev,
            Sort::CreatedOn { reverse } => *reverse = rev,
        }
    }
}

impl Default for Sort {
    fn default() -> Self {
        Self::Name { reverse: true }
    }
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Sortable for Sort {
    fn as_str(&self) -> &str {
        match self {
            Sort::Name { reverse: _ } => "Name",
            Sort::Count { reverse: _ } => "Count",
            Sort::Time { reverse: _ } => "Time",
            Sort::CreatedOn { reverse: _ } => "CreatedOn",
        }
    }
}

impl From<Sort> for &'static str {
    fn from(val: Sort) -> Self {
        match val {
            Sort::Name { reverse: _ } => "Name",
            Sort::Count { reverse: _ } => "Count",
            Sort::Time { reverse: _ } => "Time",
            Sort::CreatedOn { reverse: _ } => "CreatedOn",
        }
    }
}

#[component]
pub(crate) fn SidebarContent() -> impl IntoView {
    let session = expect_context::<RwSignal<UserSession>>();
    let sidebar = expect_context::<page_context::PageContext>().sidebar;
    let store = expect_context::<RwSignal<CountableStore>>();
    let selection = expect_context::<Memo<Selection>>();
    let screen = hooks::use_screen();
    let message = use_message();

    let is_selected = move |key: CountableId| selection.get().contains(&key);

    let action = ServerAction::<api::CreateCountable>::new();

    Effect::new(move |_| match action.value().get() {
        Some(Ok(countables)) => store.update(|s| countables.into_iter().for_each(|c| s.insert(c))),
        Some(Err(err)) => {
            message.server_err(err);
        }
        None => (),
    });

    let row_children = move |countable| view! { <TreeRow countable /> }.into_any();

    let input_ref = NodeRef::<html::Input>::new();
    let (show_search, set_show_search) = signal(false);
    let (search, set_search) = signal(None);
    let (show_sort, set_show_sort) = signal(false);
    let sort = RwSignal::new(Sort::default());

    let search_height = move || if show_search.get() { "100px" } else { "0px" };
    let sort_height = Signal::derive(move || {
        if show_sort.get() {
            "100px".to_string()
        } else {
            "0px".to_string()
        }
    });

    let handle_change = move |ev: ev::Event| {
        let value = event_target_value(&ev);

        if value.is_empty() {
            set_search.set(None)
        } else {
            set_search(Some(value))
        }
    };

    let handle_search = move |_| set_show_search.set(!show_search.get_untracked());
    let handle_sort = move |_| set_show_sort.set(!show_sort.get_untracked());

    let on_focus_out = move |_: ev::FocusEvent| {
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
            .filter(move |c| {
                c.name()
                    .to_lowercase()
                    .contains(&search.get().unwrap_or_default().to_lowercase())
            })
            .filter(move |c| c.owner_uuid() == Ok(session.get().user_uuid))
            .root_node_ids()
            .into_iter()
            .collect::<Vec<_>>();
        root.sort_by_key(|c| store.get().created_at(c));
        root.reverse();
        root.sort_by(sort.get().sort_fn(store));
        root
    };

    let children = move |c| {
        let mut children = store.get().children(&c);
        children.sort_by(sort.get().sort_fn(store));
        children
    };

    let options = vec![
        Sort::Name { reverse: true },
        Sort::Count { reverse: true },
        Sort::Time { reverse: true },
        Sort::CreatedOn { reverse: true },
    ];

    let width = move || {
        if screen.get().viewport() <= ViewPort::Small && sidebar.is_shown().get() {
            return "100vw".to_string();
        }

        format!("{}px", sidebar.width().get())
    };

    view! {
        <div class=style::sidebar style:width=width>
            <Navbar on_search=handle_search on_sort=handle_sort />
            <div
                class=move || {
                    stylance::classes!(style::search_box, show_search.get().then_some(style::shown))
                }
                style:max-height=search_height
            >
                <div>
                    <label for="search-filter" />
                    // TODO: check back later if this is resolved (change to TextField stack overflow)
                    <TextField
                        input_ref
                        id="search-filter"
                        on:blur=on_focus_out
                        on:input=handle_change
                    />
                </div>
            </div>
            <SortInputs show_sort options sort sort_height />
            <div class=style::content>
                <List each key=|c| *c children>
                    <RowSlot is_selected children=row_children slot />
                    <Separator slot>
                        <hr />
                    </Separator>
                </List>
                <ActionForm action style:padding="0px 16px">
                    <input type="hidden" name="kind" value=CountableKind::Counter.to_string() />
                    <Button
                        xstyle=xstyle!("padding": XPadding::Rect(8, 16))
                        class:hover-darken=true
                        style:width="100%"
                        attr:r#type="submit"
                        attr:aria_label="new counter"
                    >
                        <div>New Counter</div>
                    </Button>
                </ActionForm>
            </div>
        </div>
    }
}

#[component]
fn SortInputs(
    #[prop(into)] show_sort: Signal<bool>,
    #[prop(into)] sort_height: Signal<String>,
    #[prop(into)] options: Signal<Vec<Sort>>,
    sort: RwSignal<Sort>,
) -> impl IntoView {
    let handle_sort_change = move |s: Option<Sort>| {
        let mut s = s.unwrap_or_default();
        s.set_reverse(sort.get_untracked().is_reversed());
        sort.set(s);
    };

    view! {
        <div
            class=move || {
                stylance::classes!(style::sort_box, show_sort.get().then_some(style::shown))
            }
            style:max-height=sort_height
        >
            <div>
                <Button
                    xstyle=xstyle!("border-radius": XBorderRadius::Percentage(100))
                    on:click=move |_| sort.update(|s| s.reverse())
                    attr:aria_label=move || {
                        if sort.get().is_reversed() { "sort ascending" } else { "sort descending" }
                    }
                >
                    <Icon
                        kind=IconKind::Arrow
                        style:transform=move || {
                            if sort.get().is_reversed() {
                                "rotate(90deg)"
                            } else {
                                "rotate(-90deg)"
                            }
                        }
                        color=IconColor::Black
                    />
                </Button>
                <SelectField
                    id="filter-countable"
                    options=options
                    value=sort
                    on_change=handle_sort_change
                />
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
        <A href style:width="100%">
            {name}
        </A>
        <Show when=has_children>
            <ActionForm action style:margin-right="2px">
                <session::SessionFormInput session />
                <input type="hidden" name="kind" value="Phase" />
                <input type="hidden" name="parent" value=countable.0.to_string() />
                <Button
                    xstyle=xstyle!(
                        "padding": XPadding::Small, "border-radius": XBorderRadius::Percentage(100)
                    )
                    hover=ButtonHover::Darken
                    attr:r#type="submit"
                    attr:aria-label="add phase"
                >
                    <div class=style::add_phase>+</div>
                </Button>
            </ActionForm>
        </Show>
    }
}

#[component]
fn Navbar(
    #[prop(into)] on_search: EventCallback<ev::MouseEvent>,
    #[prop(into)] on_sort: EventCallback<ev::MouseEvent>,
) -> impl IntoView {
    let sidebar = expect_context::<page_context::PageContext>().sidebar;
    let screen = hooks::use_screen();

    let show_toggle_sidebar = move || screen.get().viewport() <= ViewPort::Small;

    view! {
        <nav class=style::sidebar_navbar>
            <div>
                <Show when=show_toggle_sidebar>{sidebar.toggle_button()}</Show>
                <Button
                    hover=ButtonHover::Lighten
                    xstyle=xstyle!("padding": XPadding::Medium)
                    class=style::icon

                    style:background="transparent"
                    attr:aria_label="search filter"
                    on:mousedown=on_search.clone()
                >
                    <Icon kind=IconKind::Search />
                </Button>
            </div>
            <Button
                hover=ButtonHover::Lighten
                xstyle=xstyle!("padding": XPadding::Medium)
                class=style::icon

                style:background="transparent"
                attr:aria_label="sort filter"
                on:mousedown=on_sort.clone()
            >
                <Icon kind=IconKind::Sort />
            </Button>
        </nav>
    }
}
