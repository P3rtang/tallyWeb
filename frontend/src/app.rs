#![allow(non_snake_case)]
use components::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use super::{elements::*, pages::*, preferences::ProvidePreferences, session::*, *};

pub const LEPTOS_OUTPUT_NAME: &str = env!("LEPTOS_OUTPUT_NAME");
pub const TALLYWEB_VERSION: &str = env!("TALLYWEB_VERSION");
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
        <Stylesheet href=format!("/pkg/{LEPTOS_OUTPUT_NAME}.css")/>
        <Stylesheet href="/fa/css/all.css"/>

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.svg"/>
        <Link href="https://fonts.googleapis.com/css?family=Roboto' rel='stylesheet"/>

        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="apple-mobile-web-app-capable" content="yes"/>

        <Title text="TallyWeb"/>

        <ProvideMessageSystem/>
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
                                                    <Outlet/>
                                                </ProvideCountableSignals>
                                            </ProvideStore>
                                        </ProvidePreferences>
                                    </ProvideScreenSignal>
                                </ProvideSessionSignal>
                            }
                        }
                    >

                        <Route path="" view=RouteSidebar>
                            <Route
                                path="/"
                                view=|| {
                                    view! {
                                        <Outlet/>
                                        <HomePage/>
                                    }
                                }
                            >

                                <Route path="" view=UnsetCountable/>
                                <Route path=":key" view=SetCountable/>
                            </Route>
                        </Route>
                        <Route path="/edit" view=EditWindow>
                            <Route path=":key" view=move || view! { <EditCountableWindow/> }/>
                        </Route>

                        <Route path="/preferences" view=move || view! { <PreferencesWindow/> }/>

                        <Route path="/change-username" view=move || view! { <ChangeAccountInfo/> }/>
                        <Route path="/change-password" view=ChangePassword/>
                        <Route path="/privacy-policy" view=PrivacyPolicy/>
                    </Route>
                    <TestRoutes/>
                    <Route path="/login" view=LoginPage/>
                    <Route path="/create-account" view=CreateAccount/>
                    <Route path="/*any" view=NotFound/>
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
fn RouteSidebar() -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>();
    let screen = expect_context::<Screen>();

    let sidebar_layout: Signal<SidebarLayout> = create_read_slice(screen.style, |s| (*s).into());

    let sidebar_width = create_rw_signal(400);
    provide_context(sidebar_width);

    let section_width = create_memo(move |_| {
        if show_sidebar().0 {
            format!("calc(100vw - {}px)", sidebar_width())
        } else {
            String::from("100vw")
        }
    });

    create_isomorphic_effect(move |_| {
        if screen.style.get() != ScreenStyle::Big {
            let sel_memo = create_read_slice(selection, |sel| sel.is_empty());
            sel_memo.with(|sel| show_sidebar.update(|s| *s = ShowSidebar(*sel)));
        }
    });

    let suppress_transition = create_rw_signal(false);
    let trans_class = move || (!suppress_transition()).then_some("transition-width");

    let on_resize = move |ev: ev::DragEvent| {
        if ev.client_x() as usize > SIDEBAR_MIN_WIDTH {
            suppress_transition.set(true);
            sidebar_width.update(|w| *w = ev.client_x() as usize);
        } else {
            suppress_transition.set(false);
        }
    };

    view! {
        <div style:display="flex">
            <Sidebar
                display=show_sidebar
                layout=sidebar_layout
                width=sidebar_width
                attr:class=trans_class
            >
                <SidebarContent/>
                <Show when=move || (screen.style)() != ScreenStyle::Portrait>
                    <ResizeBar
                        position=sidebar_width
                        direction=Direction::Vertical
                        on:drag=on_resize
                    />
                </Show>
            </Sidebar>
            <section style:flex-grow="1" class=trans_class style:width=section_width>
                <Outlet/>
            </section>
        </div>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let selection_signal = expect_context::<SelectionSignal>();
    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>();

    let active = create_memo(move |_| {
        selection_signal
            .get()
            .get_selected_keys()
            .into_iter()
            .copied()
            .collect()
    });

    view! {
        <div id="HomeGrid">
            <Navbar show_sidebar/>
            <InfoBox countable_list=active/>
        </div>
    }
}

#[component]
fn SidebarContent() -> impl IntoView {
    let selection_signal = expect_context::<SelectionSignal>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let sort_method = expect_context::<RwSignal<SortMethod>>();

    let show_sort_search = create_rw_signal(true);
    let show_sep = create_read_slice(preferences, |pref| pref.show_separator);

    let each = create_memo(move |_| {
        let mut root_nodes = store().root_nodes();
        root_nodes.sort_by(|a, b| {
            sort_method().sort_by()(&store.get_untracked(), &a.uuid().into(), &b.uuid().into())
        });
        root_nodes
    });

    view! {
        <nav>
            <SortSearch shown=show_sort_search/>
        </nav>
        <TreeViewWidget
            each
            key=|countable| countable.uuid()
            each_child=move |countable| {
                let key = countable.uuid().into();
                let children = create_read_slice(
                    store,
                    move |s| {
                        let mut children = s.children(&key);
                        children
                            .sort_by(|a, b| sort_method()
                                .sort_by()(
                                &store.get_untracked(),
                                &a.uuid().into(),
                                &b.uuid().into(),
                            ));
                        children
                    },
                );
                children()
            }

            view=|countable| view! { <TreeViewRow key=countable.uuid()/> }
            show_separator=show_sep
            selection_model=selection_signal
            on_click=|_, _| ()
        />

        <NewCounterButton/>
    }
}

#[component]
fn TreeViewRow(key: uuid::Uuid) -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    let data_resource = expect_context::<StateResource>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let save_handler = expect_context::<RwSignal<SaveHandlers>>();

    let key = store_value(key);

    let expand_node = move || {
        request_animation_frame(move || {
            selection.update(|s| {
                s.get_node_mut(&key()).map(|n| n.set_expand(true));
            })
        })
    };

    let click_new_phase = move |ev: ev::MouseEvent| {
        ev.stop_propagation();

        let phase_number = store.get_untracked().children(&key().into()).len();
        let name = format!("Phase {}", phase_number + 1);

        store.update(move |s| {
            let id = s.new_countable(&name, CountableKind::Phase, Some(key().into()));
            let _ = save_handler().save(
                Box::new([s.get(&id).unwrap()].to_vec()),
                Box::new(move |_| data_resource.refetch()),
            );
        });

        expand_node()
    };

    let show_context_menu = create_rw_signal(false);
    let (click_location, set_click_location) = create_signal((0, 0));
    let on_right_click = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        expect_context::<RwSignal<CloseOverlays>>().update(|_| ());
        show_context_menu.set(!show_context_menu());
        set_click_location((ev.x(), ev.y()))
    };

    let has_children = move || matches!(store().get(&key().into()), Some(Countable::Counter(_)));

    view! {
        <A href=move || key().to_string()>
            <div class="row-body" on:contextmenu=on_right_click>
                <span>{move || store().name(&key().into())}</span>
                <Show when=has_children>
                    <button on:click=click_new_phase>+</button>
                </Show>
            </div>
        </A>
        <CountableContextMenu show_overlay=show_context_menu location=click_location key=key()/>
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
    save_handlers.update(|sh| sh.connect_handler(Box::new(ServerSaveHandler::new())));
    create_effect(move |_| {
        spawn_local(async move {
            let indexed_handler = indexed::IndexedSaveHandler::new().await;
            match indexed_handler {
                Ok(ih) => {
                    let mut s = store.get_untracked();
                    if let Err(err) = ih.sync_store(&mut s).await {
                        msg.set_err(err);
                    };
                    store.set(s);
                    save_handlers.update(|sh| sh.connect_handler(Box::new(ih)))
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
