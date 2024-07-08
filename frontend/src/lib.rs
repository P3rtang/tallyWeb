#![feature(async_closure)]
#![feature(let_chains)]
#![feature(half_open_range_patterns_in_slices)]

use leptos::leptos_dom;
use wasm_bindgen::{prelude::Closure, JsCast};

pub mod app;
mod session;
pub(crate) use session::UserSession;
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

use cfg_if::cfg_if;

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
    LockMutex,
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
    #[error("Internal error converting to Any type")]
    AnyConversion,
    #[error("No Connection")]
    ConnectionError,
    #[error("{0}")]
    ServerError(String),
    #[error("Unable to get window size, Got: {0}")]
    WindowSize(String),
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

pub fn connect_on_window_resize(f: Box<dyn FnMut()>) {
    let closure = Closure::wrap(f as Box<dyn FnMut()>);
    leptos_dom::window().set_onresize(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}
