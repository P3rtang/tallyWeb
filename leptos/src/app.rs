#![allow(unused_braces)]
#![allow(non_snake_case)]
#![allow(elided_lifetimes_in_associated_constant)]
use gloo_storage::{SessionStorage, Storage};
use js_sys::Date;
use leptos::{ev::MouseEvent, *};
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::{pages::*, treeview::*};
use std::{
    any::Any,
    cmp::PartialEq,
    sync::{Arc, Mutex},
    time::Duration,
};

cfg_if::cfg_if!(
    if #[cfg(feature = "ssr")] {
        use backend::{DbCounter, DbPhase};
        impl SerCounter {
            async fn from_db(value: DbCounter) -> Self {
                let mut phase_list = Vec::new();
                for id in value.phases {
                    phase_list.push(get_phase_by_id(id).await.unwrap())
                }

                Self {
                    id: value.id,
                    name: value.name,
                    phase_list,
                }
            }
            #[allow(dead_code)]
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

        impl Into<DbPhase> for Phase {
            fn into(self) -> DbPhase {
                DbPhase {
                    id: self.id,
                    name: self.name,
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

    let id = match backend::auth::get_user(&pool, username, token, true).await {
        Ok(id) => id.id,
        Err(backend::AuthorizationError::Internal(err)) => return Err(err)?,
        Err(_) => return Ok(-1),
    };

    return Ok(id);
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
        counters.push(SerCounter::from_db(db_counter).await)
    }

    return Ok(CounterResponse::Counters(counters));
}

#[server(GetPhaseById, "/api")]
async fn get_phase_by_id(phase_id: i32) -> Result<Phase, ServerFnError> {
    let pool = backend::create_pool().await?;

    let data = backend::get_phase_by_id(&pool, phase_id).await?;
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

#[server(CreatePhase, "/api")]
async fn create_phase(name: String) -> Result<i32, ServerFnError> {
    let pool = backend::create_pool().await?;

    let id = backend::create_phase(&pool, name).await?;

    return Ok(id);
}

#[server(AssignPhase, "/api")]
async fn assign_phase(counter_id: i32, phase_id: i32) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;

    backend::assign_phase(&pool, counter_id, phase_id).await?;

    return Ok(());
}

#[server(SavePhase, "/api")]
async fn save_phase(phase: Phase) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;

    backend::update_phase(&pool, phase.into()).await?;

    return Ok(());
}

#[server(SaveAll, "/api")]
async fn save_all(user_id: i32, list: Vec<SerCounter>) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;

    for counter in list {
        backend::update_counter(&pool, counter.to_db(user_id).await).await?;
        for phase in counter.phase_list {
            backend::update_phase(&pool, phase.into()).await?;
        }
    }

    return Ok(());
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);
    let user = create_rw_signal(cx, SessionUser::default());
    provide_context(cx, user);

    view! { cx,
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet href="/pkg/tally_web.css"/>
        // <Stylesheet href="/stylers.css"/>
        <Script src="https://kit.fontawesome.com/7173474e94.js" crossorigin="anonymous"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.svg"/>

        // sets the document title
        <Title text="TallyWeb"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/login" view=LoginPage/>
                    <Route path="/create-account" view=CreateAccount/>
                    <Route path="/privacy-policy" view=PrivacyPolicy/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
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

    fn get_id(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_id()).unwrap_or_default()
    }

    fn name(&self) -> String {
        self.0.try_lock().map(|c| c.get_name()).unwrap_or_default()
    }

    fn value(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_count()).unwrap_or_default()
    }

    fn duration(&self) -> Duration {
        self.0.try_lock().map(|c| c.get_time()).unwrap_or_default()
    }

    fn progress(&self, value: i32) -> f64 {
        return 1.0 - (1.0 - 1.0_f64 / 8192.0).powi(value);
    }

    fn new_phase(&self, id: i32, name: String) -> ArcCountable {
        let _ = self.0.try_lock().map(|mut c| c.new_phase(id, name));
        self.get_children().last().cloned().unwrap()
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

impl TreeViewNodeItem<ArcCountable> for ArcCountable {
    fn into_view(self, cx: Scope) -> View {
        let node_children = expect_context::<SignalNodeChildren<ArcCountable>>(cx);
        let selection = expect_context::<Selection<ArcCountable>>(cx);

        let item = create_rw_signal(cx, self.clone());
        let get_node = create_read_slice(cx, node_children, move |node_signal| {
            node_signal.get(&item.get()).cloned().unwrap()
        });

        let click_new_phase = move |e: MouseEvent| {
            e.stop_propagation();
            create_local_resource(
                cx,
                || (),
                move |_| async move {
                    let n_phase = item
                        .get_untracked()
                        .clone()
                        .0
                        .try_lock()
                        .map(|c| c.get_phases().len() + 1)
                        .unwrap_or(1);
                    let name = format!("Phase {n_phase}");
                    let phase_id = create_phase(name.clone())
                        .await
                        .expect("Could not create Phase");
                    assign_phase(item.get_untracked().get_id(), phase_id)
                        .await
                        .expect("Could not assign phase to Counter");
                    let phase = item.get_untracked().new_phase(phase_id, name);

                    get_node.get_untracked().set_expand(true);
                    get_node.get_untracked().insert_child(cx, phase);
                },
            );
        };

        let on_right_click = move |event: MouseEvent| {
            event.prevent_default();
            log!("here")
        };

        view! { cx,
            <div on:contextmenu=on_right_click>
                <TreeViewRow node=get_node().clone() selection=selection>
                <div class="row-body">
                    <span> { self.name() } </span>
                    <Show when= move || {
                        item.get().0.try_lock().map(|c| c.has_children()).unwrap_or_default()
                    }
                    fallback= move |_| {}
                    ><button on:click=click_new_phase>+</button>
                    </Show>
                </div>
                </TreeViewRow>
            </div>
        }
        .into_view(cx)
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

impl IntoView for ArcCountable {
    fn into_view(self, cx: Scope) -> View {
        let state = expect_context::<RwSignal<CounterList>>(cx);
        let item_s = create_rw_signal(cx, self.clone());

        let (get_count, set_count) = create_slice(
            cx,
            state,
            move |_| {
                item_s
                    .get()
                    .0
                    .try_lock()
                    .map(|c| c.get_count())
                    .unwrap_or_default()
            },
            move |_, count| {
                let _ = item_s.get().0.try_lock().map(|mut c| c.set_count(count));
            },
        );

        let (get_time, _) = create_slice(
            cx,
            state,
            move |_| {
                item_s
                    .get()
                    .0
                    .try_lock()
                    .map(|c| c.get_time())
                    .unwrap_or_default()
            },
            move |_, time| {
                let _ = item_s.get().0.try_lock().map(|mut c| c.set_time(time));
            },
        );

        let on_count_click = move |e: MouseEvent| {
            e.stop_propagation();
            set_count(get_count() + 1)
        };

        view! { cx,
        <ul style="display:flex;align-items:center;flex-wrap:wrap">
            <li class="rowbox" on:click=on_count_click>
                <p class="title">{ self.name() }</p>
                <p class="info">{ move || get_count() }</p>
            </li>
            <li class="rowbox">
                <p class="title">Time</p>
                <p class="info longtime">{ move || get_time.with(|t| format_time(*t)) }</p>
            </li>
            <li class="rowbox">
                <p class="title">Progress</p>
                <Progressbar progress={
                    move || item_s.get().progress(get_count())
                } class="info">{
                    move || get_count.with(|p| format!("{:.03}%", item_s.get().progress(*p) * 100.0))
                }</Progressbar>
            </li>
        </ul>
        }
        .into_view(cx)
    }
}

fn format_time(dur: Duration) -> String {
    format!(
        "{:>02}:{:02}:{:02},{:03}",
        dur.as_secs() / 3600,
        dur.as_secs() / 60 % 60,
        dur.as_secs() % 60,
        dur.as_millis() - dur.as_secs() as u128 * 1000,
    )
}

fn timer(cx: Scope) {
    const FRAMERATE: u64 = 600;
    const INTERVAL: Duration = Duration::from_millis(1000 / FRAMERATE);

    let time = create_signal(cx, 0_u32);

    let calc_interval = |now: u32, old: u32| -> Duration {
        return if now < old {
            Duration::from_millis((1000 + now - old).into())
        } else {
            Duration::from_millis((now - old).into())
        };
    };

    create_effect(cx, move |_| {
        set_interval(
            move || {
                if let Some(selection) = use_context::<Selection<ArcCountable>>(cx) {
                    let state = use_context::<RwSignal<CounterList>>(cx);
                    if state.is_none() {
                        time.1.set(Date::new_0().get_milliseconds());
                        return;
                    }

                    if let Some(list) = state.map(|s| s.try_get()).flatten() {
                        if list.is_paused {
                            time.1.set(Date::new_0().get_milliseconds());
                            return;
                        }
                        selection
                            .get()
                            .into_iter()
                            .filter(|(_, b)| *b)
                            .for_each(|(c, _)| {
                                let item_s = create_rw_signal(cx, c.clone());

                                let (get_time, set_time) = create_slice(
                                    cx,
                                    state.unwrap(),
                                    move |_| {
                                        item_s
                                            .get()
                                            .try_lock()
                                            .map(|c| c.get_time())
                                            .unwrap_or_default()
                                    },
                                    move |_, time| {
                                        let _ =
                                            item_s.get().try_lock().map(|mut c| c.set_time(time));
                                    },
                                );

                                let interval =
                                    calc_interval(Date::new_0().get_milliseconds(), time.0.get());
                                time.1.set(Date::new_0().get_milliseconds());
                                set_time.set(get_time() + interval);
                            });
                    } else {
                        return;
                    }
                }
            },
            INTERVAL,
        )
    });
}

#[cfg(not(ssr))]
pub fn navigate(cx: Scope, page: &'static str) {
    request_animation_frame(move || {
        let navigate = leptos_router::use_navigate(cx);
        let _ = navigate(page, Default::default());
    });
}

// #[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let session_user = expect_context::<RwSignal<SessionUser>>(cx);
    create_effect(cx, move |_| {
        session_user.update(|s| s.update());
    });
    let data = create_local_resource(cx, session_user, move |user| async move {
        match get_counters_by_user_name(cx, user.username.clone(), user.token.clone()).await {
            Ok(CounterResponse::Counters(counters)) => counters,
            _ if !user.has_value() => Vec::new(),
            _ => {
                navigate(cx, "/login");
                Vec::new()
            }
        }
    });

    create_effect(cx, move |_| {});

    let list = CounterList::new(&[]);
    let state = create_rw_signal(cx, list);
    provide_context(cx, state);

    create_effect(cx, move |_| {
        let list = match data.read(cx) {
            None => state.get(),
            Some(data_list) => data_list.into(),
        };

        state.set(list)
    });

    let selection: PointerMap<ArcCountable, bool> = PointerMap::new();
    let selection_signal = create_rw_signal(cx, selection);
    provide_context(cx, selection_signal);

    timer(cx);

    window_event_listener(ev::keypress, {
        move |ev| {
            if let Some(selection) = use_context::<Selection<ArcCountable>>(cx) {
                match ev.code().as_str() {
                    "Equal" => selection.with(|list| {
                        list.into_iter().filter(|(_, b)| **b).for_each(|(node, _)| {
                            let state = expect_context::<RwSignal<CounterList>>(cx);
                            let item_s = create_rw_signal(cx, node.clone());

                            let (get_count, set_count) = create_slice(
                                cx,
                                state,
                                move |_| {
                                    item_s
                                        .get()
                                        .try_lock()
                                        .map(|c| c.get_count())
                                        .unwrap_or_default()
                                },
                                move |_, count| {
                                    let _ = item_s.get().try_lock().map(|mut c| c.set_count(count));
                                },
                            );

                            set_count(get_count() + 1);
                        })
                    }),
                    "KeyP" => {
                        let _ = state.try_update(|list| list.toggle_paused());
                    }
                    _ => (),
                }
            }
        }
    });

    let on_click = move |_| {
        let user = expect_context::<RwSignal<SessionUser>>(cx);
        user.update_untracked(|s| s.update());
        create_local_resource(
            cx,
            || (),
            async move |_| {
                let name = format!("Counter {}", state.get_untracked().list.len() + 1);
                let phase_name = String::from("Phase 1");
                let counter_id = create_counter(
                    user.get_untracked().username,
                    user.get_untracked().token,
                    name.clone(),
                )
                .await
                .map_err(|err| {
                    log!("Count not create Counter, Got: {err}");
                })
                .unwrap();
                let phase_id = create_phase(phase_name.clone())
                    .await
                    .map_err(|err| {
                        log!("Count not create Phase, Got: {err}");
                    })
                    .unwrap();
                assign_phase(counter_id, phase_id)
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

    view! { cx,
        <div id="HomeGrid">
            <TreeViewWidget start_nodes=move || { state.get().list } after=move |cx| {
                view! {cx, <button on:click=on_click class="new-counter">New Counter</button> }
            }/>
            <InfoBox/>
        </div>
    }
}

#[component]
fn Progressbar<F>(cx: Scope, progress: F, class: &'static str, children: Children) -> impl IntoView
where
    F: Fn() -> f64 + 'static,
{
    view! { cx,
        <div style="display: grid; justify-items: center" class=class>
            <div style="font-size: 0.6em; color: gray">{ children(cx) }</div>
            <progress
                max=1
                value=progress
            />
        </div>
    }
}

#[component]
fn InfoBox(cx: Scope) -> impl IntoView {
    let selection = expect_context::<Selection<ArcCountable>>(cx);
    view! { cx,
        <div id="InfoBox"> {
            move || selection.with(|list| {
                list.into_iter().filter(|(_, b)| **b).map(|(rc_counter, _)| {
                    let rc_counter_signal = create_rw_signal(cx, rc_counter.clone());
                    view! {cx, <InfoBoxRow counter=rc_counter_signal/> }
                }).collect_view(cx)})
        } </div>
    }
}

#[component]
fn InfoBoxRow(cx: Scope, counter: RwSignal<ArcCountable>) -> impl IntoView {
    view! { cx,
        <div class="row">
            <div> { counter } </div>
        </div>
    }
}

pub trait Countable: std::fmt::Debug + Send + Any {
    fn get_id(&self) -> i32;
    fn get_name(&self) -> String;
    fn get_count(&self) -> i32;
    fn set_count(&mut self, count: i32);
    fn add_count(&mut self, count: i32);
    fn get_time(&self) -> Duration;
    fn set_time(&mut self, dur: Duration);
    fn add_time(&mut self, dur: Duration);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerCounter {
    id: i32,
    name: String,
    phase_list: Vec<Phase>,
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

    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn get_count(&self) -> i32 {
        return self.phase_list.iter().map(|p| p.value()).sum();
    }

    fn set_count(&mut self, count: i32) {
        let diff = self.phase_list.iter().map(|p| p.value()).sum::<i32>()
            - self.phase_list.last().map(|p| p.value()).unwrap_or(0);
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
        return self.phase_list.iter().map(|p| p.duration()).sum();
    }

    fn set_time(&mut self, time: Duration) {
        let diff = self
            .phase_list
            .iter()
            .map(|p| p.duration())
            .sum::<Duration>()
            - self
                .phase_list
                .last()
                .map(|p| p.duration())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn get_count(&self) -> i32 {
        return self.count;
    }

    fn set_count(&mut self, count: i32) {
        self.count = count;
        let clone = self.clone();
        spawn_local(async { save_phase(clone).await.unwrap() })
    }

    fn add_count(&mut self, count: i32) {
        self.count += count;
        let clone = self.clone();
        spawn_local(async { save_phase(clone).await.unwrap() })
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

    fn get_progress(&self) -> f64 {
        return 1.0 - (1.0 - 1.0_f64 / 8192.0).powi(self.get_count());
    }

    fn is_active(&self) -> bool {
        return self.is_active;
    }

    fn toggle_active(&mut self) {
        self.is_active = !self.is_active;
        let clone = self.clone();
        spawn_local(async { save_phase(clone).await.unwrap() })
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
        let clone = self.clone();
        spawn_local(async { save_phase(clone).await.unwrap() })
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
struct CounterList {
    list: Vec<ArcCountable>,
    is_paused: bool,
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

    fn toggle_paused(&mut self) {
        self.is_paused = !self.is_paused;
        let list = self.clone();
        spawn_local(async { save_all(0, list.into()).await.unwrap() })
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
    username: String,
    token: String,
}

impl SessionUser {
    pub fn has_value(&self) -> bool {
        if self.username == "" || self.token.len() != 32 {
            return false;
        } else {
            return true;
        }
    }
    pub fn update(&mut self) {
        if let Ok(user) = SessionStorage::get::<SessionUser>("user_session") {
            self.username = user.username;
            self.token = user.token;
        }
    }
}
