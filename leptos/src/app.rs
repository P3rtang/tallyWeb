#![allow(unused_braces)]
#![allow(elided_lifetimes_in_associated_constant)]
#![allow(non_snake_case)]

use crate::countable::*;
use chrono::Duration;
use gloo_storage::{LocalStorage, Storage};
use js_sys::Date;
use leptos::{ev::MouseEvent, *};
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::{elements::*, pages::*};

pub type SelectionSignal = RwSignal<SelectionModel<ArcCountable, String>>;

#[server(LoginUser, "/api", "Url", "login_user")]
pub async fn login_user(username: String, password: String) -> Result<SessionUser, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::login_user(&pool, username, password).await?;

    let session_user = SessionUser {
        username: user.username,
        token: user.token.unwrap(),
    };

    return Ok(session_user);
}

#[server(CreateAccount, "/api", "Url", "create_account")]
pub async fn create_account(
    username: String,
    password: String,
    password_repeat: String,
) -> Result<SessionUser, ServerFnError> {
    if password != password_repeat {
        return Err(backend::LoginError::InvalidPassword.into());
    }

    let pool = backend::create_pool().await?;
    let user = backend::auth::insert_user(&pool, username, password).await?;

    let session_user = SessionUser {
        username: user.username,
        token: user.token.unwrap(),
    };

    return Ok(session_user);
}

#[server(GetUserIdFromName, "/api")]
async fn get_id_from_username(
    cx: Scope,
    username: String,
    token: String,
) -> Result<i32, ServerFnError> {
    let pool = backend::create_pool().await?;

    let db_user = backend::auth::get_user(&pool, username, token, true).await?;
    return Ok(db_user.id);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterResponse {
    Counters(Vec<SerCounter>),
    InvalidUsername,
    InvalidToken,
}

#[server(GetCountersByUserName, "/api")]
async fn get_counters_by_user_name(
    cx: Scope,
    username: String,
    token: String,
) -> Result<CounterResponse, ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = match backend::auth::get_user(&pool, username, token, true).await {
        Ok(user) => user,
        Err(backend::AuthorizationError::Internal(err)) => Err(err)?,
        Err(backend::AuthorizationError::UserNotFound) => {
            return Ok(CounterResponse::InvalidUsername)
        }
        Err(_) => return Ok(CounterResponse::InvalidToken),
    };

    let data = user.get_counters(&pool).await?;

    let mut counters = Vec::new();
    for db_counter in data {
        counters.push(
            SerCounter::from_db(
                user.username.clone(),
                user.token.clone().unwrap_or_default(),
                db_counter,
            )
            .await,
        )
    }

    counters.sort_by(|a, b| a.id.cmp(&b.id));

    return Ok(CounterResponse::Counters(counters));
}

#[server(GetCounterById, "/api")]
pub async fn get_counter_by_id(
    username: String,
    token: String,
    counter_id: i32,
) -> Result<SerCounter, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username.clone(), token.clone(), true).await?;
    let data = backend::get_counter_by_id(&pool, user.id, counter_id).await?;

    return Ok(SerCounter::from_db(username, token, data).await);
}

#[server(GetPhaseById, "/api")]
pub async fn get_phase_by_id(
    username: String,
    token: String,
    phase_id: i32,
) -> Result<Phase, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token, true).await?;
    let data = backend::get_phase_by_id(&pool, user.id, phase_id).await?;
    return Ok(data.into());
}

#[server(CreateCounter, "/api")]
async fn create_counter(
    username: String,
    token: String,
    name: String,
) -> Result<i32, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token, true).await?;
    let counter_id = backend::create_counter(&pool, user.id, name).await?;

    return Ok(counter_id);
}

#[server(UpdateCounter, "/api")]
pub async fn update_counter(
    username: String,
    token: String,
    counter: SerCounter,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token, true).await?;

    backend::update_counter(&pool, counter.to_db(user.id).await).await?;
    for phase in counter.phase_list {
        backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;
    }

    return Ok(());
}

#[server(RemoveCounter, "/api")]
pub async fn remove_counter(
    username: String,
    token: String,
    counter_id: i32,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token, true).await?;
    backend::remove_counter(&pool, user.id, counter_id).await?;

    return Ok(());
}

#[server(RemovePhase, "/api")]
pub async fn remove_phase(
    username: String,
    token: String,
    phase_id: i32,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let _ = backend::auth::get_user(&pool, username, token, true).await?;
    backend::remove_phase(&pool, phase_id).await?;

    return Ok(());
}

#[server(CreatePhase, "/api")]
async fn create_phase(
    username: String,
    token: String,
    name: String,
    hunt_type: Hunttype,
) -> Result<i32, ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = backend::auth::get_user(&pool, username, token, true).await?;
    let id = backend::create_phase(&pool, user.id, name, hunt_type.into()).await?;

    return Ok(id);
}

#[server(AssignPhase, "/api")]
async fn assign_phase(
    username: String,
    token: String,
    counter_id: i32,
    phase_id: i32,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = backend::auth::get_user(&pool, username, token, true).await?;
    backend::assign_phase(&pool, user.id, counter_id, phase_id).await?;

    return Ok(());
}

#[server(SavePhase, "/api")]
pub async fn update_phase(
    username: String,
    token: String,
    phase: Phase,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = backend::auth::get_user(&pool, username, token, true).await?;
    backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;

    return Ok(());
}

#[server(SaveAll, "/api")]
async fn save_all(
    username: String,
    token: String,
    counters: Vec<SerCounter>,
    phases: Option<Vec<Phase>>,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let phases = phases.unwrap_or_default();

    let user = backend::auth::get_user(&pool, username, token, true).await?;
    for counter in counters {
        backend::update_counter(&pool, counter.to_db(user.id).await).await?;
        for phase in counter.phase_list {
            backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;
        }
    }

    for phase in phases {
        backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;
    }

    return Ok(());
}

#[server(GetUserPreferences, "/api")]
async fn get_user_preferences(
    cx: Scope,
    username: String,
    token: String,
) -> Result<Preferences, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = match backend::auth::get_user(&pool, username.clone(), token.clone(), true).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(Preferences::default());
        }
    };
    let session_user = SessionUser {
        username: user.username,
        token: user.token.unwrap_or_default(),
    };
    let prefs = match backend::DbPreferences::db_get(&pool, user.id).await {
        Ok(data) => Preferences::from_db(&session_user, data),
        Err(backend::DataError::Uninitialized) => {
            let new_prefs = Preferences::new(&session_user);
            save_preferences(username, token, new_prefs.clone()).await?;
            new_prefs
        }
        Err(err) => return Err(err)?,
    };

    return Ok(Preferences::from(prefs));
}

#[server(SavePreferences, "/api")]
pub async fn save_preferences(
    username: String,
    token: String,
    preferences: Preferences,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token, true).await?;

    let accent_color = if preferences.use_default_accent_color {
        None
    } else {
        Some(preferences.accent_color.0)
    };

    let db_prefs = backend::DbPreferences {
        user_id: user.id,
        use_default_accent_color: preferences.use_default_accent_color,
        accent_color,
        show_separator: preferences.show_separator,
    };
    db_prefs.db_set(&pool, user.id).await?;

    return Ok(());
}

impl Display for AccountAccentColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn save(cx: Scope, list: CounterList) -> Result<(), String> {
    let user = use_context::<Memo<Option<SessionUser>>>(cx).ok_or("User Context should Exist")?;
    create_action(cx, move |_: &()| {
        save_all(
            user.get_untracked().unwrap().username,
            user.get_untracked().unwrap().token,
            list.clone().into(),
            None,
        )
    })
    .dispatch(());
    return Ok(());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    None,
    Message(String),
    Error(String),
}

impl Notification {
    fn is_some(&self) -> bool {
        self != &Self::None
    }

    fn get_text(&self) -> String {
        match self {
            Notification::None => String::new(),
            Notification::Message(msg) => msg.to_string(),
            Notification::Error(msg) => msg.to_string(),
        }
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let user = create_rw_signal(cx, None::<SessionUser>);
    let user_memo = create_memo(cx, move |_| user());
    provide_context(cx, user);
    provide_context(cx, user_memo);

    let close_overlay_signal = create_rw_signal(cx, CloseOverlays());
    provide_context(cx, close_overlay_signal);

    let change_flag_buffer = create_rw_signal(cx, Vec::<ChangeFlag>::new());
    provide_context(cx, change_flag_buffer);

    create_effect(cx, move |_| {});

    let close_overlays = move |_| {
        close_overlay_signal.update(|_| ());
    };

    let pref_resources = create_resource(cx, user, async move |user| {
        if let Some(user) = user {
            get_user_preferences(cx, user.username.clone(), user.token.clone())
                .await
                .unwrap_or(Preferences::new(&user))
        } else {
            Preferences::default()
        }
    });

    let preferences = create_rw_signal(cx, Preferences::default());
    let pref_memo = create_memo(cx, move |_| preferences());
    provide_context(cx, pref_resources);
    provide_context(cx, pref_memo);
    provide_context(cx, preferences);

    let screen_layout = create_rw_signal(cx, ScreenLayout::Big);
    let show_sidebar = create_rw_signal(cx, ShowSidebar(true));
    let toggle_sidebar = move |_| show_sidebar.update(|s| s.0 = !s.0);

    provide_context(cx, screen_layout);
    provide_context(cx, show_sidebar);

    let handle_resize = move || {
        if let Some(width) = leptos_dom::window()
            .inner_width()
            .ok()
            .map(|v| v.as_f64())
            .flatten()
        {
            if width < 1200.0 {
                screen_layout.set(ScreenLayout::Small)
            } else {
                screen_layout.set(ScreenLayout::Big);
                show_sidebar.update(|s| s.0 = true);
            }
        }
    };

    create_effect(cx, move |_| {
        handle_resize();
        connect_on_window_resize(Box::new(handle_resize));
    });

    let message = create_rw_signal(cx, Notification::None);
    let border_style = move || {
        "color: tomato;
        border: 2px solid tomato;"
    };
    provide_context(cx, message);

    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet href="/pkg/tally_web.css"/>
        // <Stylesheet href="/stylers.css"/>
        // <Script src="https://kit.fontawesome.com/7173474e94.js" crossorigin="anonymous"/>
        <Link href="https://fonts.googleapis.com/css?family=Roboto' rel='stylesheet"/>
        <Link href="/fa/css/all.css" rel="stylesheet"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.svg"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Title text="TallyWeb"/>

        <Router>
            <Transition
                fallback= || ()>
                { move || {
                    preferences.set(pref_resources.read(cx).unwrap_or_default());
                    create_effect(cx, move |_| {
                        request_animation_frame(move || {
                            user.set(SessionUser::from_storage(cx));
                        })
                    });
                }}
                <nav on:click=close_overlays>
                    <button id="toggle-sidebar" on:click=toggle_sidebar>
                        <i class="fa-solid fa-bars"></i>
                    </button>
                    <A href="/"><img src="/favicon.svg" width=48 height=48 alt="Home" class="tooltip-parent"/>
                        <span class="tooltip bottom">Home</span>
                    </A>
                    <AccountIcon/>
                </nav>

                <main on:click=close_overlays>
                    <Routes>
                        <Route path="" view=HomePage/>
                        <Route path="/preferences" view=move |cx| view!{ cx,
                            <PreferencesWindow layout=screen_layout/>
                        }/>

                        <Route path="/edit" view=EditWindow>
                            <Route path="counter" view=|cx| view!{ cx, <Outlet/> }>
                                <Route path=":id" view=move |cx| view! { cx,
                                    <EditCounterWindow layout=screen_layout/>
                                }/>
                            </Route>
                            <Route path="phase" view=|cx| view!{ cx, <Outlet/> }>
                                <Route path=":id" view=move |cx| view! { cx,
                                    <EditPhaseWindow layout=screen_layout/>
                                }/>
                            </Route>
                        </Route>

                        <Route path="/login" view=LoginPage/>
                        <Route path="/create-account" view=CreateAccount/>
                        <Route path="/privacy-policy" view=PrivacyPolicy/>
                        <Route path="/*any" view=NotFound/>
                    </Routes>

                    <Show
                        when=move || { message().is_some() }
                            fallback=|_| ()
                    >
                        <b class="notification-box" style=border_style>{ move || { message().get_text() } }</b>
                    </Show>
                </main>
            </Transition>
        </Router>
    }
}

/// 404 - Not Found
#[component]
fn NotFound(cx: Scope) -> impl IntoView {
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
        let resp = expect_context::<leptos_actix::ResponseOptions>(cx);
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! { cx,
        <h1>"Not Found"</h1>
    }
}

fn save_timer(
    cx: Scope,
    user: Memo<Option<SessionUser>>,
    save_flags: RwSignal<Vec<ChangeFlag>>,
    state: RwSignal<CounterList>,
) {
    // only the milliseconds function is a const function
    const INTERVAL: Duration = Duration::milliseconds(10 * 1000);

    let action = create_action(cx, move |(): &()| {
        save_all(
            user.get_untracked().unwrap_or_default().username,
            user.get_untracked().unwrap_or_default().token,
            state.get_untracked().into(),
            None,
        )
    });
    create_effect(cx, move |_| match action.value().get() {
        Some(Err(_)) => {
            let notification = expect_context::<RwSignal<Notification>>(cx);
            notification.set(Notification::Error(
                "Could not save Counter\n
                    Closing the tab will result in data loss"
                    .into(),
            ))
        }
        _ => {}
    });

    create_effect(cx, move |_| {
        set_interval(
            move || {
                let mut do_save = false;
                for change in save_flags.get_untracked() {
                    match change {
                        ChangeFlag::ChangeCountable(_) => do_save = true,
                        ChangeFlag::ChangePreferences(preferences) => {
                            create_action(cx, move |_: &()| {
                                save_preferences(
                                    user.get_untracked().unwrap_or_default().username,
                                    user.get_untracked().unwrap_or_default().token,
                                    preferences.clone(),
                                )
                            })
                            .dispatch(());
                        }
                    }
                }

                if do_save {
                    action.dispatch(());
                }

                save_flags.update(|s| s.clear());
            },
            INTERVAL
                .to_std()
                .unwrap_or(std::time::Duration::from_millis(30)),
        )
    });
}

fn timer(cx: Scope, selection: RwSignal<Vec<RwSignal<ArcCountable>>>) {
    const FRAMERATE: i64 = 30;
    const INTERVAL: Duration = Duration::milliseconds(1000 / FRAMERATE);

    let time = create_signal(cx, 0_u32);

    let calc_interval = |now: u32, old: u32| -> Duration {
        return if now < old {
            Duration::milliseconds((1000 + now - old).into())
        } else {
            Duration::milliseconds((now - old).into())
        };
    };

    let state = use_context::<RwSignal<CounterList>>(cx);

    create_effect(cx, move |_| {
        set_interval(
            move || {
                if state.is_none() {
                    return;
                }
                if state
                    .unwrap()
                    .try_get()
                    .map(|s| !s.is_paused)
                    .unwrap_or_default()
                {
                    let interval = calc_interval(Date::new_0().get_milliseconds(), time.0.get());
                    selection.get_untracked().iter().for_each(|c| {
                        c.update(|c| c.add_time(interval));
                    });
                }
                time.1.try_set(Date::new_0().get_milliseconds());
            },
            INTERVAL
                .to_std()
                .unwrap_or(std::time::Duration::from_millis(30)),
        )
    });
}

#[cfg(not(ssr))]
pub fn navigate(cx: Scope, page: impl ToString) {
    let page = page.to_string();
    create_effect(cx, move |_| {
        let page = page.clone();
        request_animation_frame(move || {
            let navigate = leptos_router::use_navigate(cx);
            let _ = navigate(page.as_str(), Default::default());
        });
    })
}

fn connect_keys(
    cx: Scope,
    state: RwSignal<CounterList>,
    active: RwSignal<Vec<RwSignal<ArcCountable>>>,
    save_flags: RwSignal<Vec<ChangeFlag>>,
) {
    window_event_listener(ev::keypress, move |ev| match ev.code().as_str() {
        "Equal" => {
            active.try_get().map(|a| {
                a.into_iter().for_each(|c| {
                    let _ = c.update(|c| {
                        c.add_count(1);
                        save_flags.update(|s| s.push(ChangeFlag::ChangeCountable(c.clone())))
                    });
                });
                state.try_update(|s| s.is_paused = false);
            });
        }
        "Minus" => {
            active.try_get().map(|a| {
                a.into_iter().for_each(|c| {
                    let _ = c.update(|c| {
                        c.add_count(-1);
                        save_flags.update(|s| s.push(ChangeFlag::ChangeCountable(c.clone())))
                    });
                });
                state.try_update(|s| s.is_paused = false);
            });
        }
        "KeyP" => {
            state.try_update(|s| s.toggle_paused(cx));
        }
        _ => {}
    });
}

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let session_user = expect_context::<Memo<Option<SessionUser>>>(cx);
    let user = expect_context::<RwSignal<Option<SessionUser>>>(cx);
    let change_flags = expect_context::<RwSignal<Vec<ChangeFlag>>>(cx);

    let data = create_resource(cx, session_user, move |user| async move {
        if let Some(user) = user {
            match get_counters_by_user_name(cx, user.username.clone(), user.token.clone()).await {
                Ok(CounterResponse::Counters(counters)) => counters,
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        }
    });

    provide_context(cx, data);
    let list = CounterList::new(&[]);
    let state = create_rw_signal(cx, list);
    provide_context(cx, state);

    let preferences = expect_context::<RwSignal<Preferences>>(cx);
    let accent_color = create_read_slice(cx, preferences, |prefs| prefs.accent_color.0.clone());

    let selection = SelectionModel::<ArcCountable, String>::new(
        Some(accent_color),
        create_rw_signal(cx, Vec::new()),
    );
    let selection_signal = create_rw_signal(cx, selection);
    provide_context(cx, selection_signal);

    connect_keys(
        cx,
        state,
        selection_signal.get_untracked().get_selected(),
        change_flags,
    );

    save_timer(cx, session_user, change_flags, state);
    timer(cx, selection_signal.get_untracked().get_selected());

    let on_click = move |_| {
        let user = expect_context::<RwSignal<Option<SessionUser>>>(cx);
        if user().is_none() {
            return;
        }
        create_local_resource(
            cx,
            || (),
            async move |_| {
                let name = format!("Counter {}", state.get_untracked().list.len() + 1);
                let phase_name = String::from("Phase 1");
                let counter_id = create_counter(
                    user.get_untracked().unwrap().username,
                    user.get_untracked().unwrap().token,
                    name.clone(),
                )
                .await
                .map_err(|err| match err {
                    ServerFnError::ServerError(msg) => {
                        if msg == "Provided Token is Invalid" {
                            navigate(cx, "/login")
                        }
                    }
                    _ => {}
                })
                .unwrap();

                let phase_id = create_phase(
                    user.get_untracked().unwrap().username,
                    user.get_untracked().unwrap().token,
                    phase_name.clone(),
                    Hunttype::NewOdds,
                )
                .await
                .map_err(|err| {
                    log!("Count not create Phase, Got: {err}");
                })
                .unwrap();
                assign_phase(
                    user.get_untracked().unwrap().username,
                    user.get_untracked().unwrap().token,
                    counter_id,
                    phase_id,
                )
                .await
                .map_err(|err| log!("Could not assign phase to Counter, Got: {err}"))
                .expect("Could not assign phase to Counter");

                state.update(|list| {
                    let counter = list.new_counter(counter_id, name.clone()).unwrap();
                    counter.try_lock().unwrap().new_phase(phase_id, phase_name);
                });
            },
        );
    };

    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>(cx);
    let screen_layout = expect_context::<RwSignal<ScreenLayout>>(cx);

    let selct_slice = create_read_slice(cx, selection_signal, |sel| sel.get_selected());

    let show_sep = create_read_slice(cx, preferences, |pref| pref.show_separator);

    view! { cx,
        <Show
            when=move || session_user().is_some() && data.read(cx).is_some()
            fallback=move |cx| {
                create_effect(cx, move |_| {
                    request_animation_frame(move || {
                        user.set(SessionUser::from_storage(cx));
                    })
                });
                view!{ cx, <LoadingScreen/> }
            }
        >
            { move || {
                let list = match data.read(cx) {
                    None => state.get(),
                    Some(data_list) => data_list.into(),
                };
                state.set(list)
            }}
            <div id="HomeGrid">
                { move || {
                    if screen_layout() == ScreenLayout::Small {
                        selection_signal.with(|sel| show_sidebar.update(|s| s.0 = sel.is_empty()))
                    }
                }}
                <Sidebar class="sidebar" display=show_sidebar layout=screen_layout>
                    <TreeViewWidget
                        each=move || { state.get().list }
                        key=|countable| countable.get_uuid()
                        each_child=|countable| countable.get_children()
                        view=|cx, node| {
                            view! { cx, <TreeViewRow node=node/> }
                        }
                        show_separator=show_sep
                        selection_model=selection_signal
                    />
                    <button on:click=on_click class="new-counter">New Counter</button>
                </Sidebar>
                <InfoBox countable_list=selct_slice()/>
            </div>
        </Show>
    }
}

#[component]
pub fn Progressbar<F>(
    cx: Scope,
    progress: F,
    class: &'static str,
    children: ChildrenFn,
) -> impl IntoView
where
    F: Fn() -> f64 + Copy + 'static,
{
    let preferences = expect_context::<RwSignal<Preferences>>(cx);
    let accent_color = create_read_slice(cx, preferences, |prefs| prefs.accent_color.0.clone());
    view! { cx,
        <div
            class={ format!("{class} progress-bar") }
            style="
                display:flex;
                justify-content: center
                align-items: center">
            <div
                style={ format!("
                    font-size: 1.4rem;
                    color: #BBB;
                    padding: 0px 12px;
                    margin: auto;",
                    // accent_color().0,
                )}>
                { children(cx) }
            </div>
            <div
                class="through"
                style="
                    background: #DDD;
                    padding: 1px;
                    width: 100%;
                    height: 18px;"
            >
                <Show
                    when=move || {progress() > 0.0}
                    fallback=|_| ()
                >
                <div
                    class="progress"
                    style={ move || { format!("
                        height: 18px;
                        width: max({}%, 24px);
                        background: {};
                        ",
                        progress() * 100.0,
                        accent_color(),
                    )}}>
                </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn TreeViewRow(cx: Scope, node: RwSignal<TreeNode<ArcCountable, String>>) -> impl IntoView {
    let user = expect_context::<Memo<Option<SessionUser>>>(cx);

    let countable = create_read_slice(cx, node, |node| node.row.get_untracked());

    let click_new_phase = move |e: MouseEvent| {
        e.stop_propagation();
        if user().is_none() {
            return;
        }
        create_local_resource(
            cx,
            || (),
            move |_| async move {
                let n_phase = countable
                    .get_untracked()
                    .clone()
                    .0
                    .try_lock()
                    .map(|c| c.get_phases().len() + 1)
                    .unwrap_or(1);
                let name = format!("Phase {n_phase}");
                let phase_id = create_phase(
                    user.get_untracked().unwrap().username,
                    user.get_untracked().unwrap().token,
                    name.clone(),
                    countable().get_hunt_type(),
                )
                .await
                .expect("Could not create Phase");
                assign_phase(
                    user.get_untracked().unwrap().username,
                    user.get_untracked().unwrap().token,
                    countable.get_untracked().get_id(),
                    phase_id,
                )
                .await
                .expect("Could not assign phase to Counter");

                expect_context::<RwSignal<CounterList>>(cx).update(|_| {
                    let phase = countable.get_untracked().new_phase(phase_id, name);
                    node.get_untracked().set_expand(true);
                    node.get_untracked().insert_child(cx, phase);
                });
            },
        );
    };

    let show_context_menu = create_rw_signal(cx, false);
    let (click_location, set_click_location) = create_signal(cx, (0, 0));
    let on_right_click = move |ev: MouseEvent| {
        ev.prevent_default();
        show_context_menu.set(!show_context_menu());
        set_click_location((ev.x(), ev.y()))
    };

    let has_children = countable
        .get_untracked()
        .0
        .try_lock()
        .map(|i| i.has_children())
        .unwrap_or_default();
    let id = countable
        .get_untracked()
        .0
        .try_lock()
        .map(|i| i.get_id())
        .unwrap_or(-1);

    view! { cx,
    <div
        class="row-body"
        on:contextmenu=on_right_click
        >
        <span> { move || countable().get_name() } </span>
        <Show
            when= move || {
                countable.get().0.try_lock().map(|c| c.has_children()).unwrap_or_default()
            }
            fallback= move |_| {}
        >
            <button on:click=click_new_phase>+</button>
        </Show>

    </div>
    <CountableContextMenu
        show_overlay=show_context_menu
        location=click_location
        countable_id=id
        is_phase=!has_children
    />
    }
}

// #[component]
// fn InfoBox(cx: Scope) -> impl IntoView {
//     let selection = expect_context::<SelectionSignal<ArcCountable>>(cx);
//     view! { cx,
//         <div id="InfoBox"> {
//             move || selection.with(|list| {
//                 list.selection.clone().into_iter().filter(|(_, b)| *b).map(|(rc_counter, _)| {
//                     let rc_counter_signal = create_rw_signal(cx, rc_counter.clone());
//                     view! {cx, <InfoBoxRow counter=rc_counter_signal/> }
//                 }).collect_view(cx)})
//         } </div>
//     }
// }

// #[component]
// fn InfoBoxRow(cx: Scope, counter: RwSignal<ArcCountable>) -> impl IntoView {
//     view! { cx,
//         <div class="row">
//             <div> { counter } </div>
//         </div>
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct CounterList {
    pub list: Vec<ArcCountable>,
    pub is_paused: bool,
}

impl CounterList {
    fn new(counters: &[Counter]) -> Self {
        return CounterList {
            list: counters
                .into_iter()
                .map(|c| ArcCountable::new(Box::new(c.clone())))
                .collect(),
            is_paused: true,
        };
    }

    fn new_counter(&mut self, id: i32, name: impl ToString) -> Result<ArcCountable, String> {
        let arc_counter = ArcCountable::new(Box::new(Counter::new(id, name)?));
        self.list.push(arc_counter.clone());
        return Ok(arc_counter);
    }

    pub fn toggle_paused(&mut self, cx: Scope) {
        self.is_paused = !self.is_paused;
        let _ = save(cx, self.clone().into());
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
                    id: sc.id,
                    name: sc.name,
                    phase_list,
                };
                ArcCountable::new(Box::new(counter))
            })
            .collect();
        return Self {
            list,
            is_paused: true,
        };
    }
}

impl Into<Vec<SerCounter>> for CounterList {
    fn into(self) -> Vec<SerCounter> {
        let mut rtrn_list = Vec::new();
        for arc_c in self.list {
            if let Some(counter) = arc_c
                .lock()
                .map(|c| c.as_any().downcast_ref::<Counter>().cloned())
                .ok()
                .flatten()
            {
                rtrn_list.push(counter.clone().into())
            }
        }

        return rtrn_list;
    }
}

impl std::ops::Index<usize> for CounterList {
    type Output = ArcCountable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUser {
    pub username: String,
    pub token: String,
}

impl SessionUser {
    pub fn from_storage(cx: Scope) -> Option<Self> {
        if let Ok(Some(user)) = LocalStorage::get::<Option<SessionUser>>("user_session") {
            let (user, _) = create_signal(cx, user);
            create_local_resource(cx, user, move |user| async move {
                if get_id_from_username(cx, user.username, user.token)
                    .await
                    .is_err()
                {
                    navigate(cx, "/login");
                }
            });

            return Some(user.get_untracked());
        } else {
            let _ = LocalStorage::set("user_session", None::<SessionUser>);
            navigate(cx, "/login");
            return None;
        }
    }

    pub fn has_value(&self) -> bool {
        // TODO: make this function check the backend for validity
        if self.username == "" || self.token.len() != 32 {
            return false;
        } else {
            return true;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAccentColor(pub String);

impl AccountAccentColor {
    fn new(user: &SessionUser) -> Self {
        let mut this = Self(String::new());
        this.set_user(user);
        return this;
    }

    pub fn set_user(&mut self, user: &SessionUser) {
        let letter = user
            .username
            .to_uppercase()
            .chars()
            .next()
            .unwrap_or_default();
        let color_hex = letter_to_three_digit_hash(letter);
        self.0 = format!("#{color_hex}")
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    pub use_default_accent_color: bool,
    pub accent_color: AccountAccentColor,
    pub show_separator: bool,
}

impl Preferences {
    fn new(user: &SessionUser) -> Self {
        let accent_color = AccountAccentColor::new(user);
        return Self {
            use_default_accent_color: true,
            accent_color,
            show_separator: false,
        };
    }
}

#[cfg(feature = "ssr")]
impl Preferences {
    fn from_db(user: &SessionUser, value: backend::DbPreferences) -> Self {
        return Self {
            use_default_accent_color: value.use_default_accent_color,
            accent_color: value
                .accent_color
                .map(|c| AccountAccentColor(c))
                .unwrap_or(AccountAccentColor::new(user)),
            show_separator: value.show_separator,
        };
    }
}

#[derive(Debug, Clone)]
pub enum ChangeFlag {
    ChangeCountable(ArcCountable),
    ChangePreferences(Preferences),
}
