#![allow(dead_code)]
use super::*;
use components::MessageJar;
use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::A;

pub type SaveCountableAction = Action<(UserSession, Vec<ArcCountable>), Result<(), AppError>>;
pub type SaveHandlerCountable = SaveHandler<RwSignal<ArcCountable>>;

pub trait Savable: Clone {
    fn endpoint() -> Action<(UserSession, Vec<Self>), Result<(), AppError>>
    where
        Self: Sized;
}

pub async fn save_countables(
    user: UserSession,
    countables: Vec<ArcCountable>,
) -> Result<(), AppError> {
    let mut counters = Vec::<SerCounter>::new();
    let mut phases = Vec::<Phase>::new();
    for countable in countables {
        match countable.kind() {
            CountableKind::Counter(_) => {
                let counter = countable
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

    if let Err(err) = api::save_multiple(user, Some(counters), Some(phases)).await {
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
pub struct SaveHandler<S>
where
    S: Savable + Copy + 'static,
{
    data: RwSignal<Vec<S>>,
    interval: chrono::Duration,
    is_offline: RwSignal<bool>,
}

impl<S> SaveHandler<S>
where
    S: Savable + Copy + 'static,
{
    pub fn new() -> Self {
        Self {
            data: Vec::new().into(),
            interval: chrono::Duration::minutes(2),
            is_offline: false.into(),
        }
    }

    pub fn change_save_interval(self, interval: chrono::Duration) -> Self {
        if interval < chrono::Duration::zero() {
            self
        } else {
            Self { interval, ..self }
        }
    }

    pub fn init_timer(self, user: UserSession) -> Self {
        set_interval(
            move || {
                if !self.data.get_untracked().is_empty() {
                    self.save(user.clone())
                }
            },
            self.interval.to_std().unwrap(),
        );
        self
    }

    pub fn save(&self, user: UserSession) {
        let data = self.data.get_untracked().into_iter().collect::<Vec<_>>();
        S::endpoint().dispatch((user, data))
    }

    pub fn add_countable(&self, savable: S) {
        self.data.update(|d| d.push(savable))
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
    let message = expect_context::<MessageJar>();

    let on_offline = move |_| {
        is_offline.set(true);
        message.clear();
        let _ = save_to_browser();
    };

    view! {
        <b>{err.to_string()}</b>
        <button on:click=on_offline>Go Offline</button>
        <Show when=move || err != AppError::Connection>
            <A href="/login">
                <button>Login</button>
            </A>
        </Show>
    }
}

#[component]
pub fn AskOfflineData(data: Vec<SerCounter>) -> impl IntoView {
    let state = expect_context::<RwSignal<app::CounterList>>();
    let message = expect_context::<MessageJar>();
    let user = expect_context::<RwSignal<UserSession>>();
    let save_handler = expect_context::<SaveHandlerCountable>();

    let load_data = move |_| {
        state.update(|list| list.load_offline(data.clone()));
        message.clear();
        save_handler.save(user.get_untracked())
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
