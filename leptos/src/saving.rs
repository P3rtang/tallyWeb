#![allow(dead_code)]
use std::collections::HashMap;

use super::*;
use components::Message;
use gloo_storage::{LocalStorage, Storage};
use leptos::*;

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
        expect_context::<Message>().set_server_err(&err.to_string());
        save_to_browser().await?
    }

    Ok(())
}

async fn save_to_browser() -> Result<(), gloo_storage::errors::StorageError> {
    let save_data: Vec<SerCounter> = expect_context::<RwSignal<app::CounterList>>()().into();
    LocalStorage::set("save_data", save_data)
}

#[derive(Clone, Copy)]
pub struct SaveHandler {
    data: RwSignal<Vec<ChangeFlag>>,
    user: Memo<Option<app::SessionUser>>,
    interval: chrono::Duration,
}

impl SaveHandler {
    pub fn new(user: Memo<Option<app::SessionUser>>) -> Self {
        return Self {
            data: Vec::new().into(),
            user,
            interval: chrono::Duration::minutes(2),
        };
    }

    pub fn change_save_interval(self, interval: chrono::Duration) -> Self {
        if interval < chrono::Duration::zero() {
            self
        } else {
            Self { interval, ..self }
        }
    }

    pub fn init_timer(self) -> Self {
        self.data
            .with(|_| set_timeout(move || self.save(), self.interval.to_std().unwrap()));

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
                expect_context::<SaveCountableAction>()
                    .dispatch((user, counters.values().cloned().collect()))
            }

            self.data.update(|d| d.clear());
        }
    }

    pub fn add_countable(&self, countable: ArcCountable) {
        self.data
            .update(|d| d.push(ChangeFlag::ChangeCountable(countable)))
    }
}
