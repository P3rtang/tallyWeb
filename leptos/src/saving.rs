#![allow(dead_code)]
use super::*;
use components::Message;
use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::A;
use std::collections::HashMap;

pub type SaveCountableAction = Action<(app::SessionUser, Vec<ArcCountable>), Result<(), AppError>>;

#[derive(Debug, Clone)]
pub enum ChangeFlag {
    ChangeCountable(ArcCountable),
    ChangePreferences(app::Preferences),
}

#[server(SaveMultiple, "/api")]
pub async fn save_multiple(
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

pub async fn save_countables(
    user: app::SessionUser,
    countables: Vec<ArcCountable>,
) -> Result<(), AppError> {
    let mut counters = Vec::<SerCounter>::new();
    let mut phases = Vec::<Phase>::new();
    for countable in countables {
        match countable.kind() {
            CountableKind::Counter(_) => {
                let counter = countable
                    .0
                    .try_lock()
                    .map_err(|_| AppError::LockMutex)?
                    .as_any()
                    .downcast_ref::<Counter>()
                    .ok_or(AppError::Internal)?
                    .clone()
                    .into();
                counters.push(counter)
                // update_counter(user.username.clone(), user.token.clone(), counter).await
            }
            CountableKind::Phase(_) => {
                let phase = countable
                    .0
                    .try_lock()
                    .map_err(|_| AppError::LockMutex)?
                    .as_any()
                    .downcast_ref::<Phase>()
                    .ok_or(AppError::Internal)?
                    .clone();
                phases.push(phase)
                // update_phase(user.username.clone(), user.token.clone(), phase).await
            }
        }
    }

    if let Err(err) = save_multiple(
        user.username.clone(),
        user.token.clone(),
        counters,
        Some(phases),
    )
    .await
    {
        return match err {
            ServerFnError::Request(_) => Err(AppError::Connection),
            ServerFnError::ServerError(_) => Err(AppError::Authentication),
            _ => Err(AppError::Internal),
        };
    }

    Ok(())
}

pub fn save_to_browser() -> Result<(), gloo_storage::errors::StorageError> {
    let save_data: Vec<SerCounter> = expect_context::<RwSignal<app::CounterList>>()
        .get_untracked()
        .into();
    LocalStorage::set("save_data", save_data)
}

#[derive(Clone, Copy)]
pub struct SaveHandler {
    data: RwSignal<Vec<ChangeFlag>>,
    user: Memo<Option<app::SessionUser>>,
    save_action: SaveCountableAction,
    interval: chrono::Duration,
    is_offline: RwSignal<bool>,
}

impl SaveHandler {
    pub fn new(user: Memo<Option<app::SessionUser>>) -> Self {
        let message = expect_context::<Message>();

        let save_action = create_action(
            |(user, countables): &(app::SessionUser, Vec<ArcCountable>)| {
                save_countables(user.clone(), countables.clone())
            },
        );

        let is_offline = create_rw_signal(false);

        create_effect(move |_| {
            save_action.value().with(|v| match v {
                Some(Ok(_)) if !is_offline() => {
                    message
                        .with_timeout(chrono::Duration::seconds(2))
                        .set_success_view(components::SavingSuccess);
                    LocalStorage::delete("save_data");
                }
                Some(Ok(_)) => {
                    is_offline.set(false);
                    message.set_success("Connection Restored");
                    LocalStorage::delete("save_data");
                }
                Some(Err(err)) if !is_offline() => {
                    let err = *err;
                    message
                        .without_timeout()
                        .set_err_view(move || view! { <SavingError err is_offline/> });
                }
                Some(Err(_)) => {
                    let _ = save_to_browser();
                }
                None if save_action.pending().get_untracked() => message
                    .without_timeout()
                    .set_msg_view(components::SavingMessage),
                _ => {}
            })
        });

        Self {
            data: Vec::new().into(),
            user,
            save_action,
            interval: chrono::Duration::minutes(2),
            is_offline,
        }
    }

    pub fn change_save_interval(self, interval: chrono::Duration) -> Self {
        if interval < chrono::Duration::zero() {
            self
        } else {
            Self { interval, ..self }
        }
    }

    pub fn init_timer(self) -> Self {
        set_interval(
            move || {
                if !self.data.get_untracked().is_empty() {
                    self.save()
                }
            },
            self.interval.to_std().unwrap(),
        );
        self
    }

    pub fn save(&self) {
        if let Some(user) = self.user.get_untracked() {
            let mut counters = HashMap::<String, ArcCountable>::new();
            for change in self.data.get_untracked() {
                match change {
                    ChangeFlag::ChangeCountable(c) => {
                        counters.insert(c.get_uuid(), c.clone());
                    }
                    ChangeFlag::ChangePreferences(_) => {}
                }
            }

            if !counters.is_empty() {
                self.save_action
                    .dispatch((user, counters.values().cloned().collect()));
            }

            self.data.update(|d| d.clear());
        } else {
            let _ = save_to_browser();
        }
    }

    pub fn add_countable(&self, countable: ArcCountable) {
        self.data
            .update(|d| d.push(ChangeFlag::ChangeCountable(countable)))
    }

    pub fn is_offline(&self) -> bool {
        (self.is_offline)()
    }

    pub fn set_offline(&self, is_offline: bool) {
        self.is_offline.set(is_offline)
    }
}

#[component]
pub fn SavingError(err: AppError, is_offline: RwSignal<bool>) -> impl IntoView {
    let message = expect_context::<Message>();

    let on_offline = move |_| {
        is_offline.set(true);
        message.clear();
        let _ = save_to_browser();
    };

    view! {
        <b>{ err.to_string() }</b>
        <button on:click=on_offline>Go Offline</button>
        <Show
            when=move || err != AppError::Connection
        >
            <A href="/login"><button>Login</button></A>
        </Show>
    }
}

#[component]
pub fn AskOfflineData(data: Vec<SerCounter>) -> impl IntoView {
    let state = expect_context::<RwSignal<app::CounterList>>();
    let message = expect_context::<Message>();
    let user = expect_context::<Memo<Option<app::SessionUser>>>();
    let save_handler = expect_context::<SaveHandler>();

    let load_data = move |_| {
        state.update(|list| list.load_offline(data.clone()));
        message.clear();

        if let Some(user) = user() {
            save_handler
                .save_action
                .dispatch((user, state().get_items()))
        }
    };

    let delete = move |_| {
        message.clear();
        LocalStorage::delete("save_data");
    };

    view! {
        <b>Offline data found</b>

        <button on:click=load_data>Load data</button>
        <button on:click=move |_| message.clear()>Ignore</button>
        <button on:click=delete>Delete</button>
    }
}
