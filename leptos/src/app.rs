#![allow(unused_braces)]
#![allow(elided_lifetimes_in_associated_constant)]
#![allow(non_snake_case)]

use gloo_storage::{LocalStorage, Storage};
use js_sys::Date;
use leptos::{ev::MouseEvent, *};
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::{elements::*, pages::*};
use std::{
    any::Any,
    fmt::Display,
    sync::{Arc, Mutex},
    time::Duration,
};

pub type SelectionSignal = RwSignal<SelectionModel<ArcCountable, String>>;

cfg_if::cfg_if!(
    if #[cfg(feature = "ssr")] {
        use backend::{DbCounter, DbPhase};
        impl SerCounter {
            async fn from_db(username: String, token: String, value: DbCounter) -> Self {
                let mut phase_list = Vec::new();
                for id in value.phases {
                    if let Ok(phase) = get_phase_by_id(username.clone(), token.clone(), id).await {
                        phase_list.push(phase)
                    }
                }

                Self {
                    id: value.id,
                    name: value.name,
                    phase_list,
                }
            }
            async fn to_db(&self, user_id: i32) -> DbCounter {
                DbCounter {
                    id: self.id,
                    user_id,
                    name: self.name.clone(),
                    phases: self.phase_list.iter().map(|p| p.id).collect()
                }
            }
        }

        impl From<DbPhase> for Phase {
            fn from(value: DbPhase) -> Self {
                Self {
                    id: value.id,
                    name: value.name,
                    count: value.count,
                    time: Duration::from_millis(value.time as u64),
                    is_active: false,
                }
            }
        }

        impl Phase {
            async fn to_db(&self, user_id: i32) -> DbPhase {
                DbPhase {
                    id: self.id,
                    user_id,
                    name: self.name.clone(),
                    count: self.count,
                    time: self.time.as_millis() as i64,
                }
            }
        }
    }
);

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
async fn create_phase(username: String, token: String, name: String) -> Result<i32, ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = backend::auth::get_user(&pool, username, token, true).await?;
    let id = backend::create_phase(&pool, user.id, name).await?;

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
    list: Vec<SerCounter>,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = backend::auth::get_user(&pool, username, token, true).await?;
    for counter in list {
        backend::update_counter(&pool, counter.to_db(user.id).await).await?;
        for phase in counter.phase_list {
            backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;
        }
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
            save_preferences(
                username,
                token,
                Some(new_prefs.use_default_accent_color),
                Some(new_prefs.accent_color.0.clone()),
            )
            .await?;
            new_prefs
        }
        Err(err) => return Err(err)?,
    };

    return Ok(Preferences::from(prefs));
}

#[server(SavePreferences, "/api")]
async fn save_preferences(
    username: String,
    token: String,
    use_default_accent_color: Option<bool>,
    accent_color: Option<String>,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token, true).await?;

    let db_prefs = backend::DbPreferences {
        user_id: user.id,
        use_default_accent_color: use_default_accent_color.is_some(),
        accent_color,
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
    spawn_local(async move {
        save_all(
            user.get_untracked().unwrap().username,
            user.get_untracked().unwrap().token,
            list.into(),
        )
        .await
        .unwrap()
    });
    return Ok(());
}

fn save_counter(cx: Scope, counter: SerCounter) -> Result<(), String> {
    let user = use_context::<Memo<Option<SessionUser>>>(cx).ok_or("User Context should Exist")?;
    spawn_local(async move {
        update_counter(
            user.get_untracked().unwrap().username,
            user.get_untracked().unwrap().token,
            counter,
        )
        .await
        .unwrap()
    });

    return Ok(());
}

fn save_phase(cx: Scope, phase: Phase) -> Result<(), String> {
    let user = use_context::<Memo<Option<SessionUser>>>(cx).ok_or("User Context should Exist")?;
    spawn_local(async move {
        update_phase(
            user.get_untracked().unwrap().username,
            user.get_untracked().unwrap().token,
            phase,
        )
        .await
        .unwrap()
    });

    return Ok(());
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let user = create_rw_signal(cx, None::<SessionUser>);
    let user_memo = create_memo(cx, move |_| user());
    provide_context(cx, user);
    provide_context(cx, user_memo);

    let close_overlay_signal = create_rw_signal(cx, CloseOverlays::new());
    provide_context(cx, close_overlay_signal);

    let close_overlays = move |_| {
        debug_warn!("Closing Overlays");
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

    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet href="/pkg/tally_web.css"/>
        // <Stylesheet href="/stylers.css"/>
        // <Script src="https://kit.fontawesome.com/7173474e94.js" crossorigin="anonymous"/>
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

#[derive(Debug, Clone)]
pub struct ArcCountable(Arc<Mutex<Box<dyn Countable>>>);

impl ArcCountable {
    fn new(countable: Box<dyn Countable>) -> Self {
        Self(Arc::new(Mutex::new(countable)))
    }

    pub fn get_uuid(&self) -> String {
        self.0.try_lock().map(|c| c.get_uuid()).unwrap_or_default()
    }

    fn get_id(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_id()).unwrap_or_default()
    }

    pub fn get_name(&self) -> String {
        self.0.try_lock().map(|c| c.get_name()).unwrap_or_default()
    }

    pub fn get_count(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_count()).unwrap_or_default()
    }

    pub fn add_count(&self, count: i32) {
        let _ = self.0.try_lock().map(|mut c| c.add_count(count));
    }

    pub fn get_time(&self) -> Duration {
        self.0.try_lock().map(|c| c.get_time()).unwrap_or_default()
    }

    pub fn add_time(&self, dur: Duration) {
        let _ = self.0.try_lock().map(|mut c| c.add_time(dur));
    }

    pub fn get_progress(&self) -> f64 {
        self.0
            .try_lock()
            .map(|c| c.get_progress())
            .unwrap_or_default()
    }

    fn new_phase(&self, id: i32, name: String) -> ArcCountable {
        let _ = self.0.try_lock().map(|mut c| c.new_phase(id, name));
        self.get_children().last().cloned().unwrap()
    }

    fn save(&self, cx: Scope) -> Result<(), String> {
        let has_ch = self
            .0
            .try_lock()
            .map_err(|_| "Failed to lock ArcCountable")?
            .has_children();
        if has_ch {
            save_counter(
                cx,
                self.0
                    .try_lock()
                    .unwrap()
                    .as_any()
                    .downcast_ref::<Counter>()
                    .unwrap()
                    .clone()
                    .into(),
            )
        } else {
            save_phase(
                cx,
                self.0
                    .try_lock()
                    .unwrap()
                    .as_any()
                    .downcast_ref::<Phase>()
                    .unwrap()
                    .clone(),
            )
        }
    }

    fn get_children(&self) -> Vec<ArcCountable> {
        self.0
            .try_lock()
            .map_or_else(
                |_| Vec::new(),
                |c| c.get_phases().into_iter().map(|p| p.clone()).collect(),
            )
            .clone()
    }
}

impl PartialEq for ArcCountable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.0, &*other.0)
    }
}

impl std::ops::Deref for ArcCountable {
    type Target = Mutex<Box<dyn Countable>>;

    fn deref(&self) -> &Self::Target {
        return &*self.0;
    }
}

fn timer(cx: Scope, selection: RwSignal<Vec<RwSignal<ArcCountable>>>) {
    const FRAMERATE: u64 = 30;
    const INTERVAL: Duration = Duration::from_millis(1000 / FRAMERATE);

    let time = create_signal(cx, 0_u32);

    let calc_interval = |now: u32, old: u32| -> Duration {
        return if now < old {
            Duration::from_millis((1000 + now - old).into())
        } else {
            Duration::from_millis((now - old).into())
        };
    };

    let state = use_context::<RwSignal<CounterList>>(cx);

    create_effect(cx, move |_| {
        if state.is_none() {
            return;
        }
        set_interval(
            move || {
                if let Some(list) = state.unwrap().try_get() {
                    let _ = save(cx, list);
                }
            },
            Duration::from_secs(20),
        );
    });

    create_effect(cx, move |_| {
        set_interval(
            move || {
                if state.is_some_and(|s| !s().is_paused) {
                    let interval = calc_interval(Date::new_0().get_milliseconds(), time.0.get());
                    selection.get_untracked().iter().for_each(|c| {
                        c.update(|c| c.add_time(interval));
                    });
                }
                time.1.set(Date::new_0().get_milliseconds())
            },
            INTERVAL,
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
) {
    window_event_listener(ev::keypress, move |ev| match ev.code().as_str() {
        "Equal" => {
            active.try_get().map(|a| {
                a.into_iter().for_each(|c| {
                    let _ = c.update(|c| c.add_count(1));
                    let _ = c.get_untracked().save(cx);
                });
                state.try_update(|s| s.is_paused = false);
            });
        }
        "Minus" => {
            active.try_get().map(|a| {
                a.into_iter().for_each(|c| {
                    let _ = c.update(|c| c.add_count(-1));
                    let _ = c.get_untracked().save(cx);
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

    connect_keys(cx, state, selection_signal.get_untracked().get_selected());
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

    view! { cx,
    <Suspense
        fallback=move || view!{ cx, <LoadingScreen/> }
    >
        { move || {
            let list = match data.read(cx) {
                None => state.get(),
                Some(data_list) => data_list.into(),
            };
            state.set(list)
        }}
        <Show
            when=move || session_user().is_some() && data.read(cx).is_some()
            fallback=move |cx| view!{ cx, <LoadingScreen/> }
        >
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
                        selection_model=selection_signal
                    />
                    <button on:click=on_click class="new-counter">New Counter</button>
                </Sidebar>
                <InfoBox countable_list=selct_slice()/>
            </div>
        </Show>
    </Suspense>
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

pub trait Countable: std::fmt::Debug + Send + Any {
    fn get_id(&self) -> i32;
    fn get_uuid(&self) -> String;
    fn get_name(&self) -> String;
    fn get_count(&self) -> i32;
    fn set_count(&mut self, count: i32);
    fn add_count(&mut self, count: i32);
    fn get_time(&self) -> Duration;
    fn set_time(&mut self, dur: Duration);
    fn add_time(&mut self, dur: Duration);
    fn rem_time(&mut self, dur: Duration);
    fn get_progress(&self) -> f64;
    fn is_active(&self) -> bool;
    fn toggle_active(&mut self);
    fn set_active(&mut self, active: bool);

    fn new_phase(&mut self, id: i32, name: String);
    fn new_counter(&mut self, id: i32, name: String) -> Result<ArcCountable, String>;

    fn get_phases(&self) -> Vec<&ArcCountable>;
    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable>;

    fn has_children(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SerCounter {
    id: i32,
    pub name: String,
    pub phase_list: Vec<Phase>,
}

impl From<Counter> for SerCounter {
    fn from(value: Counter) -> Self {
        let mut phase_list = Vec::new();
        for arc_p in value.phase_list {
            if let Some(phase) = arc_p
                .lock()
                .map(|c| c.as_any().downcast_ref::<Phase>().cloned())
                .ok()
                .flatten()
            {
                phase_list.push(phase.clone())
            }
        }
        return SerCounter {
            id: value.id,
            name: value.name,
            phase_list,
        };
    }
}

impl Countable for SerCounter {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_uuid(&self) -> String {
        format!("c{}", self.id)
    }

    fn get_name(&self) -> String {
        todo!()
    }

    fn get_count(&self) -> i32 {
        self.phase_list.iter().map(|p| p.get_count()).sum()
    }

    fn set_count(&mut self, count: i32) {
        let diff = self.phase_list.iter().map(|p| p.get_count()).sum::<i32>()
            - self.phase_list.last().map(|p| p.get_count()).unwrap_or(0);
        self.phase_list
            .last_mut()
            .map(|p| p.set_count(count - diff));
    }

    fn add_count(&mut self, _count: i32) {
        todo!()
    }

    fn get_time(&self) -> Duration {
        self.phase_list.iter().map(|p| p.get_time()).sum()
    }

    fn set_time(&mut self, _dur: Duration) {
        todo!()
    }

    fn add_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            p.add_time(dur);
        });
    }

    fn rem_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            p.rem_time(dur);
        });
    }

    fn get_progress(&self) -> f64 {
        todo!()
    }

    fn is_active(&self) -> bool {
        todo!()
    }

    fn toggle_active(&mut self) {
        todo!()
    }

    fn set_active(&mut self, _active: bool) {
        todo!()
    }

    fn new_phase(&mut self, _id: i32, _name: String) {
        todo!()
    }

    fn new_counter(&mut self, _id: i32, _name: String) -> Result<ArcCountable, String> {
        todo!()
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        todo!()
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        todo!()
    }

    fn has_children(&self) -> bool {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Counter {
    id: i32,
    name: String,
    phase_list: Vec<ArcCountable>,
}

#[allow(dead_code)]
impl Counter {
    fn new(id: i32, name: impl ToString) -> Result<Self, String> {
        return Ok(Counter {
            id,
            name: name.to_string(),
            phase_list: Vec::new(),
        });
    }
}

impl Countable for Counter {
    fn get_id(&self) -> i32 {
        return self.id;
    }

    fn get_uuid(&self) -> String {
        format!("c{}", self.id)
    }

    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn get_count(&self) -> i32 {
        return self.phase_list.iter().map(|p| p.get_count()).sum();
    }

    fn set_count(&mut self, count: i32) {
        let diff = self.phase_list.iter().map(|p| p.get_count()).sum::<i32>()
            - self.phase_list.last().map(|p| p.get_count()).unwrap_or(0);
        self.phase_list
            .last_mut()
            .map(|p| p.0.try_lock().map(|mut p| p.set_count(count - diff)));
    }

    fn add_count(&mut self, count: i32) {
        self.phase_list.last_mut().map(|p| {
            let _ = p.0.try_lock().map(|mut p| p.add_count(count));
        });
    }

    fn get_time(&self) -> Duration {
        return self.phase_list.iter().map(|p| p.get_time()).sum();
    }

    fn set_time(&mut self, time: Duration) {
        let diff = self
            .phase_list
            .iter()
            .map(|p| p.get_time())
            .sum::<Duration>()
            - self
                .phase_list
                .last()
                .map(|p| p.get_time())
                .unwrap_or_default();
        self.phase_list
            .last_mut()
            .map(|p| p.0.lock().map(|mut p| p.set_time(time - diff)));
    }

    fn add_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            let _ = p.0.try_lock().map(|mut p| p.add_time(dur));
        });
    }

    fn rem_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            let _ = p.0.try_lock().map(|mut p| p.rem_time(dur));
        });
    }

    fn get_progress(&self) -> f64 {
        return 1.0 - (1.0 - 1.0_f64 / 8192.0).powi(self.get_count());
    }

    fn is_active(&self) -> bool {
        for p in self.phase_list.iter() {
            if p.try_lock().map(|p| p.is_active()).unwrap_or_default() {
                return true;
            }
        }
        return false;
    }

    fn toggle_active(&mut self) {
        if self.is_active() {
            self.set_active(false)
        } else {
            self.set_active(true)
        }
    }

    fn set_active(&mut self, active: bool) {
        if !active {
            self.phase_list.iter().for_each(|p| {
                let _ = p.0.lock().map(|mut p| p.set_active(false));
            });
        } else {
            self.phase_list
                .last_mut()
                .map(|p| p.try_lock().ok())
                .flatten()
                .map(|mut p| p.set_active(active));
        }
    }

    fn new_phase(&mut self, id: i32, name: String) {
        self.phase_list
            .push(ArcCountable::new(Box::new(Phase::new(id, name))))
    }

    fn new_counter(&mut self, id: i32, name: String) -> Result<ArcCountable, String> {
        let arc_counter = ArcCountable::new(Box::new(Counter::new(id, name)?));
        self.phase_list.push(arc_counter.clone());
        return Ok(arc_counter);
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        self.phase_list.iter().collect()
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        return self.phase_list.iter_mut().collect();
    }

    fn has_children(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Phase {
    pub id: i32,
    pub name: String,
    pub count: i32,
    pub time: Duration,
    pub is_active: bool,
}

impl Phase {
    fn new(id: i32, name: impl ToString) -> Self {
        return Phase {
            id,
            name: name.to_string(),
            count: 0,
            time: Duration::ZERO,
            is_active: false,
        };
    }
}

impl Countable for Phase {
    fn get_id(&self) -> i32 {
        return self.id;
    }

    fn get_uuid(&self) -> String {
        format!("p{}", self.id)
    }

    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn get_count(&self) -> i32 {
        return self.count;
    }

    fn set_count(&mut self, count: i32) {
        self.count = count;
    }

    fn add_count(&mut self, count: i32) {
        self.count += count;
    }

    fn get_time(&self) -> Duration {
        return self.time;
    }

    fn set_time(&mut self, dur: Duration) {
        self.time = dur
    }

    fn add_time(&mut self, dur: Duration) {
        self.time += dur
    }

    fn rem_time(&mut self, dur: Duration) {
        self.time -= dur
    }

    fn get_progress(&self) -> f64 {
        return 1.0 - (1.0 - 1.0_f64 / 8192.0).powi(self.get_count());
    }

    fn is_active(&self) -> bool {
        return self.is_active;
    }

    fn toggle_active(&mut self) {
        self.is_active = !self.is_active;
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    fn new_phase(&mut self, _: i32, _: String) {
        return ();
    }

    fn new_counter(&mut self, _: i32, _: String) -> Result<ArcCountable, String> {
        return Err(String::from("Can not add counter to phase"));
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        return vec![];
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        return vec![];
    }

    fn has_children(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

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
}

impl Preferences {
    fn new(user: &SessionUser) -> Self {
        let accent_color = AccountAccentColor::new(user);
        return Self {
            use_default_accent_color: true,
            accent_color,
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
        };
    }
}
