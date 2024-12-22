#![allow(non_snake_case)]
use components::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use super::{elements::*, pages::*, preferences::ProvidePreferences, session::*, *};

pub const SIDEBAR_MIN_WIDTH: usize = 280;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let close_overlay_signal = create_rw_signal(CloseOverlays());
    provide_context(close_overlay_signal);

    let close_overlays = move |_| {
        close_overlay_signal.update(|_| ());
    };

    let show_sidebar = create_rw_signal(ShowSidebar(true));
    provide_context(show_sidebar);

    view! {
        <Stylesheet href=format!("/pkg/{LEPTOS_OUTPUT_NAME}.css") />
        <Stylesheet href="/fa/css/all.css" />

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.svg" />
        <Link href="https://fonts.googleapis.com/css?family=Roboto' rel='stylesheet" />

        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Meta name="mobile-web-app-capable" content="yes" />

        <Title text="TallyWeb" />

        <ProvideMessageSystem />
        <Router>
            <main on:click=close_overlays>
                <Routes>
                    <Route
                        path=""
                        ssr=SsrMode::Async
                        view=|| {
                            view! {
                                <ProvideSessionSignal>
                                    <ProvideScreenSignal>
                                        <ProvidePreferences>
                                            <ProvideStore>
                                                <ProvideCountableSignals>
                                                    <Outlet />
                                                </ProvideCountableSignals>
                                            </ProvideStore>
                                        </ProvidePreferences>
                                    </ProvideScreenSignal>
                                </ProvideSessionSignal>
                            }
                        }
                    >

                        <Route
                            path="/"
                            view=|| {
                                view! {
                                    <Outlet />
                                    <HomePage />
                                }
                            }
                        >

                            <Route path="" view=UnsetCountable />
                            <Route path=":key" view=SetCountable />
                        </Route>
                        <Route path="/edit" view=EditWindow>
                            <Route path=":key" view=move || view! { <EditCountableWindow /> } />
                        </Route>

                        <Route path="/preferences" view=move || view! { <PreferencesWindow /> }>
                            <Route path="styling" view=move || view! { <StylingPreferences /> } />
                            <Route path="account" view=move || view! { <AccountPreferences /> } />
                            <Route path="misc" view=move || view! { <MiscPreferences /> } />
                            <Route
                                path=""
                                view=move || {
                                    let nav = use_navigate();
                                    nav("preferences/styling", Default::default());
                                }
                            />
                        </Route>

                        <Route
                            path="/change-username"
                            view=move || view! { <ChangeAccountInfo /> }
                        />
                        <Route path="/change-password" view=ChangePassword />
                    </Route>
                    <TestRoutes />
                    <Route path="/login" view=LoginPage />
                    <Route path="/create-account" view=CreateAccount />
                    <Route path="/*any" view=NotFound />
                </Routes>
            </main>
        </Router>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! { <h1>"Not Found"</h1> }
}

#[component]
fn HomeSidebar(width: MaybeSignal<usize>) -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>();
    let screen = expect_context::<Screen>();

    let sidebar_layout: Signal<SidebarLayout> = create_read_slice(screen.style, |s| (*s).into());

    create_isomorphic_effect(move |_| {
        if screen.style.get() != ScreenStyle::Big {
            let sel_memo = create_read_slice(selection, |sel| sel.is_empty());
            sel_memo.with(|sel| show_sidebar.update(|s| *s = ShowSidebar(*sel)));
        }
    });

    view! {
        <Sidebar display=show_sidebar layout=sidebar_layout width>
            <SidebarContent />
        </Sidebar>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>();
    let selection_signal = expect_context::<SelectionSignal>();
    let preferences = expect_context::<RwSignal<Preferences>>();

    let accent = create_read_slice(preferences, |p| {
        Color::try_from(p.accent_color.clone().0.as_str()).unwrap_or_default()
    });

    let show_body_border = create_read_slice(preferences, |p| p.show_body_border);

    let active = create_memo(move |_| {
        selection_signal
            .get()
            .get_selected_keys()
            .into_iter()
            .copied()
            .collect()
    });

    let sidebar: Box<dyn Fn(MaybeSignal<usize>) -> Fragment> =
        Box::new(move |width| view! { <HomeSidebar width /> }.into());

    let navbar: Box<dyn Fn() -> Fragment> = Box::new(move || {
        let on_close_sidebar: std::rc::Rc<dyn Fn(bool)> =
            std::rc::Rc::new(move |show| show_sidebar.set(ShowSidebar(show)));
        view! { <Navbar show_sidebar=Signal::derive(move || show_sidebar().0) on_close_sidebar /> }
            .into()
    });

    let show_sidebar = Signal::derive(move || show_sidebar().0);

    view! {
        <Page sidebar navbar accent show_sidebar show_body_border>
            <InfoBox countable_list=active />
        </Page>
    }
}

#[component(transparent)]
fn SidebarContent() -> impl IntoView {
    let selection_signal = expect_context::<SelectionSignal>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let sort_method = expect_context::<RwSignal<SortMethod>>();

    let show_sort_search = create_rw_signal(true);
    let show_sep = create_read_slice(preferences, |pref| pref.show_separator);
    let search = create_rw_signal(String::new());
    provide_context(search);

    let each = create_memo(move |_| {
        let mut root_nodes = store()
            .filter(move |c| c.name().to_lowercase().contains(&search().to_lowercase()))
            .raw_filter(move |c| !c.is_archived())
            .root_nodes();
        root_nodes
            .sort_by(|a, b| sort_method().sort_by()(&store(), &a.uuid().into(), &b.uuid().into()));
        root_nodes
    });

    #[allow(clippy::single_match)]
    let on_sort_key = move |ev: ev::KeyboardEvent| match ev.key().as_str() {
        "Enter" => {
            let mut nodes = store()
                .raw_filter(move |c| c.name().to_lowercase().contains(&search().to_lowercase()))
                .nodes();
            nodes.sort_by(|a, b| {
                sort_method().sort_by()(&store.get_untracked(), &a.uuid().into(), &b.uuid().into())
            });
            if let Some(first) = nodes.first() {
                leptos_router::use_navigate()(&first.uuid().to_string(), Default::default());
            }
        }
        _ => {}
    };

    let each_child = move |countable: &Countable| {
        let key = countable.uuid().into();
        let children = create_read_slice(store, move |s| {
            let mut children = s.children(&key);
            children.sort_by(|a, b| sort_method().sort_by()(s, a, b));
            children
                .into_iter()
                .map(|c| store.get_untracked().get(&c))
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default()
        });
        children()
    };

    view! {
        <nav>
            <SortSearch shown=show_sort_search search on_keydown=on_sort_key />
        </nav>
        <TreeViewWidget
            each
            key=|countable| countable.uuid()
            each_child
            view=|countable| view! { <TreeViewRow key=countable.uuid() /> }
            show_separator=show_sep
            selection_model=selection_signal
            on_click=|_, _| ()
        />

        <NewCounterButton />
    }
}

#[component]
fn TreeViewRow(key: uuid::Uuid) -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    let data_resource = expect_context::<StateResource>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let save_handler = expect_context::<RwSignal<SaveHandlers>>();
    let search = expect_context::<RwSignal<String>>();

    let expand_node = move |key: uuid::Uuid, expand: bool| {
        selection.update(|s| {
            if let Some(node) = s.get_node_mut(&key) {
                node.set_expand(expand)
            }
        })
    };

    let includes_search = create_memo(move |_| {
        !search().is_empty()
            && store
                .get_untracked()
                .name(&key.into())
                .to_lowercase()
                .contains(&search().to_lowercase())
    });
    let selected = create_memo(move |_| selection().is_selected(&key));
    let parent = store.get_untracked().parent(&key.into());

    create_isomorphic_effect(move |_| {
        if let Some(p) = parent {
            if includes_search() || selected() {
                expand_node(p.into(), true)
            }
        }
    });

    let click_new_phase = move |ev: ev::MouseEvent| {
        ev.stop_propagation();

        let phase_number = store.get_untracked().children(&key.into()).len();
        let name = format!("Phase {}", phase_number + 1);

        store.update(move |s| {
            let id = s.new_countable(&name, CountableKind::Phase, Some(key.into()));
            let _ = save_handler().save(
                Box::new([s.get(&id).unwrap()].to_vec()),
                Box::new(move |_| data_resource.refetch()),
            );
        });

        request_animation_frame(move || expand_node(key, true))
    };

    let show_context_menu = create_rw_signal(false);
    let (click_location, set_click_location) = create_signal((0, 0));
    let on_right_click = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        expect_context::<RwSignal<CloseOverlays>>().update(|_| ());
        show_context_menu.set(!show_context_menu());
        set_click_location((ev.x(), ev.y()))
    };

    let has_children = move || matches!(store().get(&key.into()), Some(Countable::Counter(_)));

    let search_split = create_memo(move |_| {
        if search().is_empty() {
            return None;
        }
        let name = store().name(&key.into());
        if let Some(idx) = name.to_lowercase().find(&search().to_lowercase()) {
            let (first, rest) = name.split_at(idx);
            let (_, last) = rest.split_at(search().len());
            Some((first.to_string(), last.to_string()))
        } else {
            None
        }
    });

    view! {
        <A href=move || key.to_string()>
            <div class="row-body" on:contextmenu=on_right_click>
                <Show
                    when=move || search_split().is_some()
                    fallback=move || view! { <span>{move || store().name(&key.into())}</span> }
                >
                    <div>
                        <span>{move || search_split().unwrap().0}</span>
                        <span style:background="var(--accent)" style:color="black">
                            {search}
                        </span>
                        <span>{move || search_split().unwrap().1}</span>
                    </div>
                </Show>
                <Show when=has_children>
                    <button on:click=click_new_phase>+</button>
                </Show>
            </div>
        </A>
        <CountableContextMenu show_overlay=show_context_menu location=click_location key />
    }
}

#[component]
fn SetCountable() -> impl IntoView {
    #[derive(Debug, PartialEq, Params, Clone)]
    struct Key {
        key: String,
    }

    let selection = expect_context::<SelectionSignal>();

    let key_memo = create_memo(move |old_key| {
        let new_key = use_params::<Key>()
            .get()
            .ok()
            .and_then(|p| uuid::Uuid::parse_str(&p.key).ok());

        if let Some(key) = new_key
            && old_key != Some(&new_key)
        {
            selection.update(|sel| sel.select(&key));
        }

        new_key
    });

    create_isomorphic_effect(move |_| key_memo.track());
}

#[component]
fn UnsetCountable() -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    selection.update(|sel| sel.clear_selection())
}

#[component(transparent)]
fn ProvideCountableSignals(children: ChildrenFn) -> impl IntoView {
    let msg = expect_context::<MessageJar>();
    let store = expect_context::<RwSignal<CountableStore>>();

    let selection = SelectionModel::<uuid::Uuid, Countable>::new();
    let selection_signal = create_rw_signal(selection);
    provide_context(selection_signal);

    provide_context(create_rw_signal(SortMethod::default()));

    let save_handlers = create_rw_signal(SaveHandlers::new());

    let server_handler = Box::new(ServerSaveHandler::new());
    save_handlers.update(|sh| sh.connect_handler(server_handler.clone()));

    // when the page closes, gets minimized or navigated away from save the store
    window_event_listener(ev::blur, move |_| {
        if let Err(err) = save_handlers
            .get_untracked()
            .save(Box::new(store), Box::new(|_| ()))
        {
            msg.set_err(err)
        }
    });

    create_effect(move |_| {
        spawn_local(async move {
            let indexed_handler = indexed::IndexedSaveHandler::new().await;
            match indexed_handler {
                Ok(ih) => {
                    let mut s = store.get_untracked();
                    if let Err(err) = ih.sync_store(&mut s).await {
                        msg.set_err(err);
                    };
                    if let Err(err) = save_handlers
                        .get_untracked()
                        .save(Box::new(s.clone()), Box::new(|_| ()))
                    {
                        msg.set_err(err);
                    }
                    store.set(s.clone());
                    save_handlers.update(|sh| sh.connect_handler(Box::new(ih)));
                    if let Err(err) = save_handlers
                        .get_untracked()
                        .save(Box::new(store), Box::new(|_| ()))
                    {
                        msg.set_err(err)
                    }
                }
                Err(err) => msg.set_msg(format!(
                    "Local saving could not be initialised\nGot error: {}",
                    err
                )),
            }
        })
    });
    provide_context(save_handlers);

    children()
}
