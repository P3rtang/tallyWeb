#![allow(non_snake_case)]
use chrono::Duration;
use components::*;
use js_sys::Date;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use super::{elements::*, pages::*, preferences::ProvidePreferences, session::*, *};

pub const LEPTOS_OUTPUT_NAME: &str = env!("LEPTOS_OUTPUT_NAME");
pub const TALLYWEB_VERSION: &str = env!("TALLYWEB_VERSION");
pub const SIDEBAR_MIN_WIDTH: usize = 280;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterResponse {
    Counters(Vec<SerCounter>),
    InvalidUsername,
    InvalidToken,
}

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

    let save_handler = SaveHandlerCountable::new();
    provide_context(save_handler);

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
                                            <ProvideCountableSignals>
                                                <ProvideStore>
                                                    <Outlet/>
                                                </ProvideStore>
                                            </ProvideCountableSignals>
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

fn timer(selection_signal: SelectionSignal) {
    const FRAMERATE: i64 = 30;
    const INTERVAL: Duration = Duration::milliseconds(1000 / FRAMERATE);

    let time = create_signal(0_u32);

    let calc_interval = |now: u32, old: u32| -> Duration {
        if now < old {
            Duration::milliseconds((1000 + now - old).into())
        } else {
            Duration::milliseconds((now - old).into())
        }
    };

    create_effect(move |_| {
        set_interval(
            move || {
                let interval = calc_interval(
                    Date::new_0().get_milliseconds(),
                    time.0.try_get().unwrap_or_default(),
                );

                selection_signal.try_update(|s| {
                    s.get_selected_keys()
                        .iter()
                        .filter_map(|key| s.get(key))
                        .filter(|c| c.is_active())
                        .for_each(|c| c.add_time(interval))
                });
                time.1.try_set(Date::new_0().get_milliseconds());
            },
            INTERVAL
                .to_std()
                .unwrap_or(std::time::Duration::from_millis(30)),
        );
    });
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
    let show_sort_search = create_rw_signal(true);

    let selection_signal = expect_context::<SelectionSignal>();
    let state = expect_context::<RwSignal<CounterList>>();
    let preferences = expect_context::<RwSignal<Preferences>>();

    let show_sep = create_read_slice(preferences, |pref| pref.show_separator);
    let state_len = create_read_slice(state, |s| s.list.len());

    view! {
        <nav>
            <SortSearch list=state shown=show_sort_search/>
        </nav>
        <TreeViewWidget
            each=move || { state.get().get_filtered_list() }
            key=|countable| countable.get_uuid()
            each_child=move |countable| {
                let mut children = countable.get_children();
                children.sort_by(state.get().sort.sort_by());
                children
            }

            view=|key| view! { <TreeViewRow key/> }
            show_separator=show_sep
            selection_model=selection_signal
            on_click=|_, _| ()
        />

        <NewCounterButton state_len/>
    }
}

#[component]
fn TreeViewRow(key: uuid::Uuid) -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    let user = expect_context::<RwSignal<UserSession>>();
    let data_resource = expect_context::<StateResource>();

    let (key, _) = create_signal(key);

    let countable = create_read_slice(selection, move |model| {
        model.get(&key.get_untracked()).cloned()
    });

    let expand_node = create_write_slice(selection, move |model, _| {
        if let Some(node) = model.get_node_mut(&key.get_untracked()) {
            node.is_expanded = true;
        }
    });

    let click_new_phase = move |e: web_sys::MouseEvent| {
        e.stop_propagation();
        create_local_resource(
            || (),
            move |_| async move {
                let n_phase = countable
                    .get_untracked()
                    .clone()
                    .map(|c| c.get_children().len())
                    .unwrap_or_default();
                let name = format!("Phase {}", n_phase + 1);
                if let Some(countable) = countable.get_untracked() {
                    let user = user.get_untracked();
                    let new_phase = Phase::new(name, countable.get_uuid(), user.user_uuid);
                    api::update_phase(user, new_phase)
                        .await
                        .expect("Could not create Phase");
                }

                data_resource.refetch();
                expand_node(());
            },
        );
    };

    let show_context_menu = create_rw_signal(false);
    let (click_location, set_click_location) = create_signal((0, 0));
    let on_right_click = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        expect_context::<RwSignal<CloseOverlays>>().update(|_| ());
        show_context_menu.set(!show_context_menu());
        set_click_location((ev.x(), ev.y()))
    };

    let has_children = countable
        .get_untracked()
        .map(|c| !c.get_children().is_empty())
        .unwrap_or_default();

    view! {
        <A href=move || key().to_string()>
            <div class="row-body" on:contextmenu=on_right_click>
                <span>
                    {move || countable.get_untracked().map(|c| c.get_name()).unwrap_or_default()}
                </span>
                <Show when=move || has_children>
                    <button on:click=click_new_phase>+</button>
                </Show>
            </div>
        </A>
        <Show when=move || countable.get_untracked().is_some()>
            <CountableContextMenu show_overlay=show_context_menu location=click_location key/>
        </Show>
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

#[component]
fn ProvideCountableSignals(children: ChildrenFn) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let save_handler = expect_context::<SaveHandlerCountable>();

    let data = create_blocking_resource(user, move |user| async move {
        match api::get_counters_by_user_name(user).await {
            Ok(api::CounterResponse::Counters(counters)) => counters,
            _ => Vec::new(),
        }
    });

    provide_context(data);

    let selection = SelectionModel::<uuid::Uuid, ArcCountable>::new();
    let selection_signal = create_rw_signal(selection);
    provide_context(selection_signal);

    timer(selection_signal);

    let state = create_rw_signal(CounterList::new(&[]));
    provide_context(state);

    view! {
        <Transition>

            {
                let list = match data.get() {
                    Some(data_list) if !save_handler.is_offline() => data_list.into(),
                    _ => state.get(),
                };
                state.set(list.clone());
                children()
            }

        </Transition>
    }
}
