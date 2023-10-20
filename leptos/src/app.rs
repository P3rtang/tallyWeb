#![allow(non_snake_case)]
use crate::countable::*;
use chrono::Duration;
use components::*;
use gloo_storage::{LocalStorage, Storage};
use js_sys::Date;
use leptos::{ev::MouseEvent, logging::log, *};
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

use crate::{elements::*, pages::*};

pub type StateResource = Resource<Option<SessionUser>, Vec<SerCounter>>;
pub type SelectionSignal = RwSignal<SelectionModel<String, ArcCountable>>;
pub type SaveAllAction = Action<(SessionUser, Vec<SerCounter>), Result<(), ServerFnError>>;
pub type SaveCountableAction = Action<(SessionUser, ArcCountable), Result<(), ServerFnError>>;

#[server(LoginUser, "/api", "Url", "login_user")]
pub async fn login_user(
    username: String,
    password: String,
    remember: Option<String>,
) -> Result<SessionUser, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::login_user(&pool, username, password, remember.is_some()).await?;

    let session_user = SessionUser {
        username: user.username,
        token: user.token.unwrap(),
    };

    Ok(session_user)
}

#[server(CreateAccount, "/api", "Url", "create_account")]
pub async fn create_account(
    username: String,
    password: String,
    password_repeat: String,
) -> Result<SessionUser, ServerFnError> {
    if password != password_repeat {
        Err(backend::LoginError::InvalidPassword)?;
    }

    let pool = backend::create_pool().await?;
    let user = backend::auth::insert_user(&pool, username, password).await?;

    let session_user = SessionUser {
        username: user.username,
        token: user.token.unwrap(),
    };

    Ok(session_user)
}

#[server(ChangePassword, "/api")]
pub async fn change_password(
    username: String,
    old_pass: String,
    new_pass: String,
    new_pass_repeat: String,
) -> Result<(), ServerFnError> {
    if new_pass != new_pass_repeat {
        Err(backend::LoginError::InvalidPassword)?;
    };

    let pool = backend::create_pool().await?;
    let _ = backend::auth::change_password(&pool, username, old_pass, new_pass).await?;

    Ok(())
}

#[server(GetUserIdFromName, "/api")]
async fn get_id_from_username(username: String, token: String) -> Result<i32, ServerFnError> {
    let pool = backend::create_pool().await?;
    let db_user = backend::auth::get_user(&pool, username, token).await?;

    Ok(db_user.id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterResponse {
    Counters(Vec<SerCounter>),
    InvalidUsername,
    InvalidToken,
}

#[server(GetCountersByUserName, "/api")]
async fn get_counters_by_user_name(
    username: String,
    token: String,
) -> Result<CounterResponse, ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = match backend::auth::get_user(&pool, username, token).await {
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

    Ok(CounterResponse::Counters(counters))
}

#[server(GetCounterById, "/api")]
pub async fn get_counter_by_id(
    username: String,
    token: String,
    counter_id: i32,
) -> Result<SerCounter, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username.clone(), token.clone()).await?;
    let data = backend::get_counter_by_id(&pool, user.id, counter_id).await?;

    Ok(SerCounter::from_db(username, token, data).await)
}

#[server(GetPhaseById, "/api")]
pub async fn get_phase_by_id(
    username: String,
    token: String,
    phase_id: i32,
) -> Result<Phase, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token).await?;
    let data = backend::get_phase_by_id(&pool, user.id, phase_id).await?;
    Ok(data.into())
}

#[server(CreateCounter, "/api")]
pub async fn create_counter(
    username: String,
    token: String,
    name: String,
) -> Result<i32, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token).await?;
    let counter_id = backend::create_counter(&pool, user.id, name).await?;

    Ok(counter_id)
}

#[server(UpdateCounter, "/api")]
pub async fn update_counter(
    username: String,
    token: String,
    counter: SerCounter,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token).await?;

    backend::update_counter(&pool, counter.to_db(user.id).await).await?;
    for phase in counter.phase_list {
        backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;
    }

    Ok(())
}

#[server(RemoveCounter, "/api")]
pub async fn remove_counter(
    username: String,
    token: String,
    counter_id: i32,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token).await?;
    backend::remove_counter(&pool, user.id, counter_id).await?;

    Ok(())
}

#[server(RemovePhase, "/api")]
pub async fn remove_phase(
    username: String,
    token: String,
    phase_id: i32,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let _ = backend::auth::get_user(&pool, username, token).await?;
    backend::remove_phase(&pool, phase_id).await?;

    Ok(())
}

#[server(CreatePhase, "/api")]
pub async fn create_phase(
    username: String,
    token: String,
    name: String,
    hunt_type: Hunttype,
) -> Result<i32, ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = backend::auth::get_user(&pool, username, token).await?;
    let id = backend::create_phase(&pool, user.id, name, hunt_type.into()).await?;

    Ok(id)
}

#[server(AssignPhase, "/api")]
pub async fn assign_phase(
    username: String,
    token: String,
    counter_id: i32,
    phase_id: i32,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = backend::auth::get_user(&pool, username, token).await?;
    backend::assign_phase(&pool, user.id, counter_id, phase_id).await?;

    Ok(())
}

#[server(SavePhase, "/api")]
pub async fn update_phase(
    username: String,
    token: String,
    phase: Phase,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;

    let user = backend::auth::get_user(&pool, username, token).await?;
    backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;

    Ok(())
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

    let user = backend::auth::get_user(&pool, username, token).await?;

    for counter in counters {
        backend::update_counter(&pool, counter.to_db(user.id).await).await?;
        for phase in counter.phase_list {
            backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;
        }
    }

    for phase in phases {
        backend::update_phase(&pool, user.id, phase.to_db(user.id).await).await?;
    }

    Ok(())
}

#[server(GetUserPreferences, "/api")]
async fn get_user_preferences(
    username: String,
    token: String,
) -> Result<Preferences, ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = match backend::auth::get_user(&pool, username.clone(), token.clone()).await {
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

    Ok(Preferences::from(prefs))
}

#[server(SavePreferences, "/api")]
pub async fn save_preferences(
    username: String,
    token: String,
    preferences: Preferences,
) -> Result<(), ServerFnError> {
    let pool = backend::create_pool().await?;
    let user = backend::auth::get_user(&pool, username, token).await?;

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
        multi_select: preferences.multi_select,
    };
    db_prefs.db_set(&pool, user.id).await?;

    Ok(())
}

impl Display for AccountAccentColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(dead_code)]
fn set_fullscreen() {
    create_effect(|_| {
        log!("{}", document().fullscreen());
        let _ = document().document_element().unwrap().request_fullscreen();
        log!("{}", document().fullscreen());
    });
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let user = SessionUser::from_storage();

    let msg = Message::new(Duration::seconds(5));
    provide_context(msg);

    let save_all = create_action(move |(user, list): &(SessionUser, Vec<SerCounter>)| {
        save_all(
            user.username.clone(),
            user.token.clone(),
            list.clone(),
            None,
        )
    });
    provide_context(save_all);

    let save_countable = create_action(|(user, countable): &(SessionUser, ArcCountable)| {
        let user = user.clone();
        let countable = countable.clone();

        async move {
            let _ = match countable.kind() {
                CountableKind::Counter(_) => {
                    update_counter(
                        user.username.clone(),
                        user.token.clone(),
                        countable
                            .0
                            .try_lock()
                            .unwrap()
                            .as_any()
                            .downcast_ref::<Counter>()
                            .unwrap()
                            .clone()
                            .into(),
                    )
                    .await
                }
                CountableKind::Phase(_) => {
                    update_phase(
                        user.username.clone(),
                        user.token.clone(),
                        countable
                            .0
                            .try_lock()
                            .unwrap()
                            .as_any()
                            .downcast_ref::<Phase>()
                            .unwrap()
                            .clone(),
                    )
                    .await
                }
            };
        }
    });

    provide_context(save_countable);

    create_effect(move |_| {
        if let Some(Err(_err)) = save_all.value().get() {
            msg.set_msg("Token Expired");
            navigate("/login")
        }
    });

    let user_memo = create_memo(move |_| user());
    provide_context(user);
    provide_context(user_memo);

    let change_flag_buffer = create_rw_signal(Vec::<ChangeFlag>::new());
    provide_context(change_flag_buffer);

    let close_overlay_signal = create_rw_signal(CloseOverlays());
    provide_context(close_overlay_signal);

    let close_overlays = move |_| {
        close_overlay_signal.update(|_| ());
    };

    let pref_resources = create_resource(user, async move |user| {
        if let Some(user) = user {
            get_user_preferences(user.username.clone(), user.token.clone())
                .await
                .unwrap_or(Preferences::new(&user))
        } else {
            Preferences::default()
        }
    });

    let preferences = create_rw_signal(Preferences::default());
    let pref_memo = create_memo(move |_| preferences());
    provide_context(pref_resources);
    provide_context(pref_memo);
    provide_context(preferences);

    let screen_layout = create_rw_signal(ScreenLayout::Big);
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
                screen_layout.set(ScreenLayout::Narrow)
            } else if width < 1200.0 {
                screen_layout.set(ScreenLayout::Small)
            } else {
                screen_layout.set(ScreenLayout::Big);
                show_sidebar.update(|s| s.0 = true);
            }
        }
    };

    let selection = SelectionModel::<String, ArcCountable>::new();
    let selection_signal = create_rw_signal(selection);
    provide_context(selection_signal);

    create_effect(move |_| {
        preferences
            .with(|pref| selection_signal.update(|sel| sel.set_multi_select(pref.multi_select)))
    });

    timer(selection_signal);

    create_effect(move |_| {
        handle_resize();
        connect_on_window_resize(Box::new(handle_resize));
    });

    view! {
        // injects a stylesheet into the document <head>
        <Stylesheet href="/pkg/tally_web.css"/>
        // <Stylesheet href="/stylers.css"/>
        // <Script src="https://kit.fontawesome.com/7173474e94.js" crossorigin="anonymous"/>
        <Link href="https://fonts.googleapis.com/css?family=Roboto' rel='stylesheet"/>
        <Link href="/fa/css/all.css" rel="stylesheet"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.svg"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="apple-mobile-web-app-capable" content="yes"/>

        <Title text="TallyWeb"/>

        <Router>
            <Transition fallback=move || view! { <LoadingScreen/> }>
                { move || {
                    preferences.set(pref_resources.get().unwrap_or_default());
                }}
            </Transition>
            <Navbar/>

            <main on:click=close_overlays>
                // on navigation clear any messages or errors from the message box
                { move || {
                    let location = use_location();
                    location.state.with(|_| msg.clear())
                }}
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/preferences" view=move || view! {
                        <PreferencesWindow layout=screen_layout/>
                    }/>

                    <Route path="/edit" view=EditWindow>
                        <Route path="counter" view=|| view!{ <Outlet/> }>
                            <Route path=":id" view=move || view! {
                                <EditCounterWindow layout=screen_layout/>
                            }/>
                        </Route>
                        <Route path="phase" view=|| view!{ <Outlet/> }>
                            <Route path=":id" view=move || view! {
                                <EditPhaseWindow layout=screen_layout/>
                            }/>
                        </Route>
                    </Route>

                    <Route path="/login" view=LoginPage/>
                    <Route path="/create-account" view=CreateAccount/>
                    <Route path="/change-username" view=move || view!{ <ChangeAccountInfo user/> }/>
                    <Route path="/change-password" view=NewPassword/>
                    <Route path="/privacy-policy" view=PrivacyPolicy/>
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

    view! {
        <h1>"Not Found"</h1>
    }
}

fn save_timer(
    user: Memo<Option<SessionUser>>,
    save_flags: RwSignal<Vec<ChangeFlag>>,
    state: RwSignal<CounterList>,
) {
    // only the milliseconds function is a const function
    const INTERVAL: Duration = Duration::milliseconds(10 * 1000);

    create_effect(move |_| {
        set_interval(
            move || {
                let mut do_save = false;
                for change in save_flags.get_untracked() {
                    match change {
                        ChangeFlag::ChangeCountable(_) => do_save = true,
                        ChangeFlag::ChangePreferences(preferences) => {
                            create_action(move |_: &()| {
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
                    expect_context::<SaveAllAction>().dispatch((user().unwrap(), state().into()));
                }

                save_flags.update(|s| s.clear());
            },
            INTERVAL
                .to_std()
                .unwrap_or(std::time::Duration::from_millis(30)),
        )
    });
}

fn timer(selection_signal: SelectionSignal) {
    const FRAMERATE: i64 = 30;
    const INTERVAL: Duration = Duration::milliseconds(1000 / FRAMERATE);

    let time = create_signal(0_u32);
    provide_context(time);

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

                selection_signal.update(|s| {
                    s.get_selected_keys()
                        .iter()
                        .map(|key| s.get(key))
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

#[cfg(not(ssr))]
pub fn navigate(page: impl ToString) {
    let page = page.to_string();
    create_effect(move |_| {
        let page = page.clone();
        request_animation_frame(move || {
            let navigate = leptos_router::use_navigate();
            navigate(page.as_str(), Default::default());
        });
    });
}

fn connect_keys(model: SelectionSignal, save_flags: RwSignal<Vec<ChangeFlag>>) {
    window_event_listener(ev::keypress, move |ev| match ev.code().as_str() {
        "Equal" => model.update(|m| {
            m.selection().into_iter().for_each(|c| {
                c.set_active(true);
                c.add_count(1);
                save_flags.update(|list| list.push(ChangeFlag::ChangeCountable(c)))
            })
        }),
        "Minus" => model.update(|m| {
            m.selection().into_iter().for_each(|c| {
                c.set_active(true);
                c.add_count(-1);
                save_flags.update(|list| list.push(ChangeFlag::ChangeCountable(c)))
            })
        }),
        "KeyP" => model.update(|list| {
            list.selection().into_iter().for_each(|c| {
                c.set_active(!c.is_active());
                save_flags.update(|list| list.push(ChangeFlag::ChangeCountable(c)))
            })
        }),
        _ => {}
    });
}

#[component]
pub fn HomePage() -> impl IntoView {
    let session_user = expect_context::<Memo<Option<SessionUser>>>();
    let change_flags = expect_context::<RwSignal<Vec<ChangeFlag>>>();

    let data = create_local_resource(session_user, move |user| async move {
        if let Some(user) = user {
            match get_counters_by_user_name(user.username.clone(), user.token.clone()).await {
                Ok(CounterResponse::Counters(counters)) => counters,
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        }
    });

    provide_context(data);
    let list = CounterList::new(&[]);
    let state = create_rw_signal(list);

    provide_context(state);

    let preferences = expect_context::<RwSignal<Preferences>>();
    let accent_color = create_read_slice(preferences, |prefs| prefs.accent_color.0.clone());

    let selection_signal = expect_context::<SelectionSignal>();

    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>();
    let screen_layout = expect_context::<RwSignal<ScreenLayout>>();

    connect_keys(selection_signal, change_flags);

    save_timer(session_user, change_flags, state);

    let show_sep = create_read_slice(preferences, |pref| pref.show_separator);
    let show_sort_search = create_rw_signal(true);
    let state_len = create_read_slice(state, |s| s.list.len());

    let active = create_read_slice(selection_signal, move |sel| {
        let mut slc = sel
            .get_selected_keys()
            .iter()
            .map(|key| sel.get(*key).clone())
            .collect::<Vec<_>>();
        slc.sort_by(state().sort.sort_by());

        slc
    });

    view! {
        <Show
            when=move || session_user().is_some() && data.get().is_some()
            fallback=move || view!{ <LoadingScreen/> }
        >
            { move || {
                let list = match data.get() {
                    None => state.get(),
                    Some(data_list) => data_list.into(),
                };
                state.set(list)
            }}
            <div id="HomeGrid">
                { move || {
                    if screen_layout() == ScreenLayout::Small {
                        let sel_memo = create_read_slice(selection_signal, |sel| sel.selection());
                        sel_memo.with(|sel| show_sidebar.update(|s| *s = ShowSidebar(sel.is_empty())));
                    }
                }}
                <Sidebar class="sidebar" display=show_sidebar layout=screen_layout>
                    <SortSearch list=state shown=show_sort_search/>
                    <TreeViewWidget
                        each=move || { state.get().get_filtered_list() }
                        key=|countable| countable.get_uuid()
                        each_child=|countable| countable.get_children()
                        view=|node| {
                            view! { <TreeViewRow node=node/> }
                        }
                        show_separator=show_sep
                        selection_model=selection_signal
                        selection_color=accent_color
                    />
                    <NewCounterButton state_len/>
                </Sidebar>
                <InfoBox countable_list=active/>
            </div>
        </Show>
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
            class={ format!("{class} progress-bar") }
            style="
                display:flex;
                justify-content: center
                align-items: center">
            <div
                style={ "
                    font-size: 1.4rem;
                    color: #BBB;
                    padding: 0px 12px;
                    margin: auto;".to_string()}>
                { children() }
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
                    fallback=|| ()
                >
                <div
                    class="progress"
                    style={ move || { format!("
                        height: 18px;
                        width: max({}%, 10px);
                        background: {};
                        ",
                        progress() * 100.0,
                        color()
                    )}}>
                </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn TreeViewRow(node: RwSignal<TreeNode<ArcCountable, String>>) -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    let user = expect_context::<Memo<Option<SessionUser>>>();
    let data = expect_context::<Resource<Option<SessionUser>, Vec<SerCounter>>>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let accent_color = create_read_slice(preferences, |pref| pref.accent_color.0.clone());

    let countable = create_read_slice(node, |node| node.row.clone());

    let click_new_phase = move |e: MouseEvent| {
        e.stop_propagation();
        if user().is_none() {
            return;
        }
        create_local_resource(
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
                    countable.get_untracked().get_hunt_type(),
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

                node.update(|n| {
                    let phase = n.row.new_phase(phase_id, name);
                    n.set_expand(true);
                    n.insert_child(phase, selection);
                });
            },
        );
    };

    let show_context_menu = create_rw_signal(false);
    let (click_location, set_click_location) = create_signal((0, 0));
    let on_right_click = move |ev: MouseEvent| {
        ev.prevent_default();
        expect_context::<RwSignal<CloseOverlays>>().update(|_| ());
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

    view! {
    <div
        class="row-body"
        on:contextmenu=on_right_click
        >
        <span> { move || countable().get_name() } </span>
        <Show
            when= move || {
                countable.get().0.try_lock().map(|c| c.has_children()).unwrap_or_default()
            }
            fallback=||()
        >
            <button on:click=click_new_phase>+</button>
        </Show>

    </div>
    <CountableContextMenu
        show_overlay=show_context_menu
        location=click_location
        counters_resource=data
        countable_id=id
        is_phase=!has_children
        accent_color
    />
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CounterList {
    pub list: HashMap<String, ArcCountable>,
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

    pub fn get_items(
        &self,
        model: SelectionModel<String, ArcCountable>,
    ) -> HashMap<String, ArcCountable> {
        model
            .items
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v.row))
            .collect()
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

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUser {
    pub username: String,
    pub token: String,
}

impl SessionUser {
    pub fn from_storage() -> RwSignal<Option<SessionUser>> {
        let user_signal = create_rw_signal(None::<SessionUser>);

        create_effect(move |_| {
            if let Ok(Some(user)) = LocalStorage::get::<Option<SessionUser>>("user_session") {
                let action = create_action(move |user: &SessionUser| {
                    get_id_from_username(user.username.clone(), user.token.clone())
                });
                action.dispatch(user.clone());

                create_effect(move |_| {
                    if let Some(Ok(id)) = action.value()() && id > 0 {
                        user_signal.set(Some(user.clone()))
                    } else if let Some(Err(_)) = action.value()() {
                        navigate("/login")
                    }
                });
            } else {
                let _ = LocalStorage::set("user_session", None::<SessionUser>);
                navigate("/login");
            }
        });

        user_signal
    }

    pub fn has_value(&self) -> bool {
        // TODO: make this function check the backend for validity
        !(self.username.is_empty() || self.token.len() != 32)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAccentColor(pub String);

impl AccountAccentColor {
    fn new(user: &SessionUser) -> Self {
        let mut this = Self(String::new());
        this.set_user(user);
        this
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
    pub multi_select: bool,
}

impl Preferences {
    fn new(user: &SessionUser) -> Self {
        let accent_color = AccountAccentColor::new(user);
        Self {
            use_default_accent_color: true,
            accent_color,
            show_separator: false,
            multi_select: false,
        }
    }
}

#[cfg(feature = "ssr")]
impl Preferences {
    fn from_db(user: &SessionUser, value: backend::DbPreferences) -> Self {
        Self {
            use_default_accent_color: value.use_default_accent_color,
            accent_color: value
                .accent_color
                .map(|c| AccountAccentColor(c))
                .unwrap_or(AccountAccentColor::new(user)),
            show_separator: value.show_separator,
            multi_select: value.multi_select,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChangeFlag {
    ChangeCountable(ArcCountable),
    ChangePreferences(Preferences),
}
