#![allow(non_snake_case)]
use chrono::Duration;
use components::*;
use js_sys::Date;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{elements::*, pages::*, preferences::ProvidePreferences, session::*, *};

pub const LEPTOS_OUTPUT_NAME: &str = env!("LEPTOS_OUTPUT_NAME");
pub const TALLYWEB_VERSION: &str = env!("TALLYWEB_VERSION");

pub type StateResource = Resource<UserSession, Vec<SerCounter>>;
pub type SelectionSignal = RwSignal<SelectionModel<uuid::Uuid, ArcCountable>>;

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

    let msg = Message::new(Duration::seconds(5));
    provide_context(msg);

    let close_overlay_signal = create_rw_signal(CloseOverlays());
    provide_context(close_overlay_signal);

    let state = create_rw_signal(CounterList::new(&[]));
    provide_context(state);

    let close_overlays = move |_| {
        close_overlay_signal.update(|_| ());
    };

    let screen_layout = create_rw_signal(SidebarStyle::Landscape);
    let show_sidebar = create_rw_signal(ShowSidebar(true));

    provide_context(screen_layout);
    provide_context(show_sidebar);

    let handle_resize = move || {
        if let Some(width) = leptos_dom::window()
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
        {
            if width < 600.0 {
                screen_layout.set(SidebarStyle::Portrait)
            } else if width < 1200.0 {
                screen_layout.set(SidebarStyle::Hover)
            } else {
                screen_layout.set(SidebarStyle::Landscape);
                show_sidebar.update(|s| s.0 = true);
            }
        }
    };

    let save_handler = SaveHandlerCountable::new();
    provide_context(save_handler);

    // create_effect(move |_| if let Some(user) = user.get() {
    //     save_handler.init_timer(user);
    // });

    // connect_keys(selection_signal, save_handler, user_memo);

    // create_effect(move |_| {
    //     preferences
    //         .with(|pref| selection_signal.update(|sel| sel.set_multi_select(pref.multi_select)))
    // });

    create_effect(move |_| {
        handle_resize();
        connect_on_window_resize(Box::new(handle_resize));
    });

    view! {
        <Stylesheet href=format!("/pkg/{LEPTOS_OUTPUT_NAME}.css")/>
        <Stylesheet href="/fa/css/all.css"/>

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.svg"/>
        <Link href="https://fonts.googleapis.com/css?family=Roboto' rel='stylesheet"/>

        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="apple-mobile-web-app-capable" content="yes"/>

        <Title text="TallyWeb"/>

        <Router>
            <main on:click=close_overlays>
                // on navigation clear any messages or errors from the message box
                {move || {
                    let location = use_location();
                    location.state.with(|_| msg.clear())
                }}
                <Routes>
                    <Route
                        path=""
                        ssr=SsrMode::Async
                        view=|| {
                            view! {
                                <ProvideSessionSignal>
                                    <ProvidePreferences/>
                                    <ProvideCountableSignals/>
                                    <Outlet/>
                                </ProvideSessionSignal>
                            }
                        }
                    >
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
                        <Route
                            path="/preferences"
                            view=move || view! { <PreferencesWindow layout=screen_layout/> }
                        />

                        <Route path="/edit" view=EditWindow>
                            <Route
                                path=":id"
                                view=move || view! { <EditCounterWindow layout=screen_layout/> }
                            />
                        </Route>

                        <Route path="/change-username" view=move || view! { <ChangeAccountInfo/> }/>
                        <Route path="/change-password" view=NewPassword/>
                        <Route path="/privacy-policy" view=PrivacyPolicy/>
                        <Route path="/*any" view=NotFound/>
                    </Route>
                    <Route path="/login" view=LoginPage/>
                    <Route path="/create-account" view=CreateAccount/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
            <MessageBox msg/>
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

#[allow(dead_code)]
fn connect_keys(
    model: SelectionSignal,
    save_handler: SaveHandlerCountable,
    user: RwSignal<UserSession>,
) {
    window_event_listener(ev::keypress, move |ev| match ev.code().as_str() {
        "Equal" => model.update(|m| {
            m.selection_mut().into_iter().for_each(|c| {
                c.set_active(true);
                c.add_count(1);
                save_handler.add_countable(c.clone().into());
            })
        }),
        "Minus" => model.update(|m| {
            m.selection_mut().into_iter().for_each(|c| {
                c.set_active(true);
                c.add_count(-1);
                save_handler.add_countable(c.clone().into());
            })
        }),
        "KeyP" => model.update(|list| {
            list.selection_mut().into_iter().for_each(|c| {
                c.set_active(!c.is_active());
                save_handler.add_countable(c.clone().into());
                save_handler.save(user());
            })
        }),
        _ => {}
    });
}

#[component]
pub fn HomePage() -> impl IntoView {
    let save_handler = expect_context::<SaveHandlerCountable>();
    let selection_signal = expect_context::<SelectionSignal>();
    let state = expect_context::<RwSignal<CounterList>>();
    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>();
    let screen_layout = expect_context::<RwSignal<SidebarStyle>>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let data = expect_context::<StateResource>();

    let accent_color = create_read_slice(preferences, |pref| pref.accent_color.0.clone());

    let active = create_read_slice(selection_signal, move |sel| {
        let mut slc = sel
            .get_selected_keys()
            .iter()
            .filter_map(|key| sel.get(*key).cloned())
            .collect::<Vec<_>>();
        slc.sort_by(state().sort.sort_by());

        slc
    });

    view! {
        <Transition fallback=move || {
            view! { <LoadingScreen/> }
        }>
            {move || {
                let list = match data.get() {
                    Some(data_list) if !save_handler.is_offline() => data_list.into(),
                    _ => state.get(),
                };
                state.set(list)
            }}
            <div id="HomeGrid">
                {move || {
                    if screen_layout() == SidebarStyle::Hover {
                        let sel_memo = create_read_slice(selection_signal, |sel| sel.is_empty());
                        sel_memo.with(|sel| show_sidebar.update(|s| *s = ShowSidebar(*sel)));
                    }
                }}
                <Sidebar display=show_sidebar layout=screen_layout accent_color=accent_color>
                    <SidebarContent/>
                </Sidebar> <section style:flex-grow="1" style:transition="width .5s">
                    <Navbar/>
                    <InfoBox countable_list=active/>
                </section>
            </div>
        </Transition>
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
    let accent_color = create_read_slice(preferences, |prefs| prefs.accent_color.0.clone());

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
            selection_color=accent_color
            on_click=|key: &uuid::Uuid, ev: leptos::ev::MouseEvent| {
                ev.stop_propagation();
                leptos_router::use_navigate()(&key.to_string(), Default::default())
            }
        />

        <NewCounterButton state_len/>
    }
}

#[component]
pub fn Progressbar<F, C>(
    progress: F,
    color: C,
    class: &'static str,
    children: ChildrenFn,
) -> impl IntoView
where
    F: Fn() -> f64 + Copy + 'static,
    C: Fn() -> &'static str + Copy + 'static,
{
    view! {
        <div
            class=format!("{class} progress-bar")
            style="
            display:flex;
            justify-content: center
            align-items: center"
        >
            <div style="
                    font-size: 1.4rem;
                    color: #BBB;
                    padding: 0px 12px;
                    margin: auto;"
                .to_string()>{children()}</div>
            <div
                class="through"
                style="
                background: #DDD;
                padding: 1px;
                width: 100%;
                height: 18px;"
            >
                <Show when=move || { progress() > 0.0 } fallback=|| ()>
                    <div
                        class="progress"
                        style=move || {
                            format!(
                                "
                        height: 18px;
                        width: max({}%, 10px);
                        background: {};
                        ",
                                progress() * 100.0,
                                color(),
                            )
                        }
                    ></div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn TreeViewRow(key: uuid::Uuid) -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    let user = expect_context::<RwSignal<UserSession>>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let accent_color = create_read_slice(preferences, |pref| pref.accent_color.0.clone());
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
        .map(|c| c.get_children().len() > 0)
        .unwrap_or_default();

    view! {
        <A href=move || key().to_string() on:click=|ev| ev.prevent_default()>
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
            <CountableContextMenu
                show_overlay=show_context_menu
                location=click_location
                key
                accent_color
            />
        </Show>
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CounterList {
    pub list: HashMap<uuid::Uuid, ArcCountable>,
    search: Option<String>,
    pub sort: SortCountable,
}

impl CounterList {
    fn new(counters: &[Counter]) -> Self {
        return CounterList {
            list: counters
                .iter()
                .map(|c| (c.get_uuid(), ArcCountable::new(Box::new(c.clone()))))
                .collect(),
            search: None,
            sort: SortCountable::Name(false),
        };
    }

    pub fn search(&mut self, value: &str) {
        self.search = Some(value.to_lowercase().to_string())
    }

    pub fn get_items(&mut self) -> Vec<ArcCountable> {
        self.list.values().cloned().collect()
    }

    pub fn get_filtered_list(&mut self) -> Vec<ArcCountable> {
        let mut list = self.list.values().cloned().collect::<Vec<_>>();

        list.sort_by(self.sort.sort_by());

        if let Some(search) = &self.search {
            let mut list_starts_with = Vec::new();
            let mut child_starts_with = Vec::new();
            let mut list_contains = Vec::new();
            let mut child_contains = Vec::new();

            for counter in list.iter() {
                let name = counter.get_name().to_lowercase();
                if name.starts_with(search) {
                    list_starts_with.push(counter.clone())
                } else if counter.has_child_starts_with(search) {
                    child_starts_with.push(counter.clone())
                } else if name.contains(search) {
                    list_contains.push(counter.clone())
                } else if counter.has_child_contains(search) {
                    child_contains.push(counter.clone())
                }
            }

            list_starts_with.append(&mut child_starts_with);
            list_starts_with.append(&mut list_contains);
            list_starts_with.append(&mut child_contains);

            list_starts_with
        } else {
            list
        }
    }

    pub fn load_offline(&mut self, data: Vec<SerCounter>) {
        let list: CounterList = data.into();
        self.list = list.list;
    }
}

impl From<Vec<SerCounter>> for CounterList {
    fn from(value: Vec<SerCounter>) -> Self {
        let list = value
            .into_iter()
            .map(|sc| {
                let phase_list: Vec<ArcCountable> = sc
                    .phase_list
                    .into_iter()
                    .map(|p| ArcCountable::new(Box::new(p)))
                    .collect();
                let counter = Counter {
                    uuid: sc.uuid,
                    owner_uuid: sc.owner_uuid,
                    name: sc.name,
                    phase_list,
                    created_at: sc.created_at,
                };
                (counter.get_uuid(), ArcCountable::new(Box::new(counter)))
            })
            .collect();
        Self {
            list,
            ..Default::default()
        }
    }
}

impl Default for CounterList {
    fn default() -> Self {
        Self::new(&[])
    }
}

impl From<CounterList> for Vec<SerCounter> {
    fn from(val: CounterList) -> Self {
        let mut rtrn_list = Vec::new();
        for arc_c in val.list.values() {
            if let Some(counter) = arc_c
                .lock()
                .map(|c| c.as_any().downcast_ref::<Counter>().cloned())
                .ok()
                .flatten()
            {
                rtrn_list.push(counter.clone().into())
            }
        }

        rtrn_list
    }
}

#[component]
fn SetCountable() -> impl IntoView {
    #[derive(Debug, PartialEq, Params, Clone)]
    struct Key {
        key: String,
    }

    let selection = expect_context::<SelectionSignal>();

    let key_memo = create_memo(move |_| {
        if let Some(key) = use_params::<Key>()
            .get()
            .map(|p| uuid::Uuid::parse_str(&p.key).ok())
            .ok()
            .flatten()
        {
            selection.update(|sel| sel.select(&key));
        }
    });

    create_isomorphic_effect(move |_| key_memo.track());
}

#[component]
fn UnsetCountable() -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    selection.update(|sel| sel.clear_selection())
}

#[component]
fn ProvideCountableSignals() -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();

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

    view! {
        <Transition fallback=move || {
            view! { <components::LoadingScreen></components::LoadingScreen> }
        }>
            {
                data.track();
            }
        </Transition>
    }
}
