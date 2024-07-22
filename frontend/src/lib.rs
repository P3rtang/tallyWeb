#![feature(async_closure)]
#![feature(let_chains)]
#![feature(half_open_range_patterns_in_slices)]

use leptos::leptos_dom;
use wasm_bindgen::{prelude::Closure, JsCast};

pub mod app;
mod session;
pub(crate) use session::SessionFormInput;
pub use session::UserSession;
mod screen;
pub(crate) use screen::{ProvideScreenSignal, Screen, ScreenStyle};
mod preferences;
pub(crate) use preferences::{PrefResource, Preferences};
mod tests;
pub(crate) use tests::*;

pub(crate) mod api;
mod countable;
pub(crate) mod elements;
mod pages;
pub(crate) mod saving;

use countable::*;
use saving::*;

mod countable_v2;
pub(crate) use countable_v2::{CountableStore, ProvideStore};

use cfg_if::cfg_if;

#[cfg(feature = "ssr")]
pub mod middleware;

cfg_if! {
    if #[cfg(feature = "hydrate")] {

        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen]
        pub fn hydrate() {
            use app::*;
            use leptos::*;

            console_error_panic_hook::set_once();

            leptos::mount_to_body(move || {
                view! { <App/> }
            });
        }
    }
}

pub type SelectionSignal = leptos::RwSignal<components::SelectionModel<uuid::Uuid, ArcCountable>>;
pub type StateResource = leptos::Resource<UserSession, Vec<SerCounter>>;

#[derive(
    Debug, Clone, PartialEq, Eq, thiserror::Error, Default, serde::Serialize, serde::Deserialize,
)]
pub enum AppError {
    #[error("Internal server Error")]
    #[default]
    Internal,
    #[error("Failed to write to browser LocalStorage")]
    SetLocalStorage,
    #[error("Connection Error")]
    Connection,
    #[error("Authentication Error")]
    Authentication,
    #[error("User is missing auth_token")]
    MissingToken,
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Invalid Username or Password")]
    InvalidSecrets,
    #[error("Invalid Password provided")]
    InvalidPassword,
    #[error("Invalid Username provided")]
    InvalidUsername,
    #[error("User data not found")]
    UserNotFound,
    #[error("Failed to lock a Countable Mutex")]
    LockMutex(String),
    #[error("Error connecting to db pool\nGot Error: {0}")]
    DbConnection(String),
    #[error("Error extracting actix web data\nGot Error: {0}")]
    Extraction(String),
    #[error("Could not get data from database\nGot Error: {0}")]
    DatabaseError(String),
    #[error("Actix web Error")]
    ActixError(String),
    #[error("Missing Session cookie")]
    MissingSession,
    #[error("Invalid Session cookie")]
    InvalidSession(String),
    #[error("Internal error converting to Any type")]
    AnyConversion,
    #[error("No Connection")]
    ConnectionError,
    #[error("{0}")]
    ServerError(String),
    #[error("Unable to get window size, Got: {0}")]
    WindowSize(String),
    #[error("Url payload error")]
    UrlPayload,
    #[error("{0}: cannot contain children")]
    CannotContainChildren(String),
}

impl From<gloo_storage::errors::StorageError> for AppError {
    fn from(_: gloo_storage::errors::StorageError) -> Self {
        Self::SetLocalStorage
    }
}

impl From<leptos::ServerFnError> for AppError {
    fn from(value: leptos::ServerFnError) -> Self {
        match value {
            leptos::ServerFnError::Request(_) => AppError::ConnectionError,
            leptos::ServerFnError::ServerError(str) => AppError::ServerError(str),
            _ => serde_json::from_str(&value.to_string())
                .unwrap_or(AppError::ServerError(value.to_string())),
        }
    }
}

impl From<serde_qs::Error> for AppError {
    fn from(_: serde_qs::Error) -> Self {
        Self::UrlPayload
    }
}

#[cfg(feature = "ssr")]
impl Into<actix_web::Error> for AppError {
    fn into(self) -> actix_web::Error {
        match self {
            AppError::MissingToken => actix_web::error::ErrorBadRequest(self),
            _ => todo!(),
        }
    }
}

impl<T> From<std::sync::TryLockError<std::sync::MutexGuard<'_, T>>> for AppError {
    fn from(value: std::sync::TryLockError<std::sync::MutexGuard<'_, T>>) -> Self {
        Self::LockMutex(value.to_string())
    }
}

impl<T> From<std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> for AppError {
    fn from(value: std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        Self::LockMutex(value.to_string())
    }
}

pub fn connect_on_window_resize(f: Box<dyn FnMut()>) {
    let closure = Closure::wrap(f as Box<dyn FnMut()>);
    leptos_dom::window().set_onresize(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}
