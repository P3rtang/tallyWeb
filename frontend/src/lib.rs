#![feature(async_closure)]
#![feature(let_chains)]
#![feature(half_open_range_patterns_in_slices)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(result_flattening)]

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
pub(crate) mod elements;
mod pages;
pub(crate) mod saving;

use saving::*;

pub mod countable;
pub(crate) use countable::*;

use cfg_if::cfg_if;

#[cfg(feature = "ssr")]
pub mod middleware;

cfg_if! {
    if #[cfg(docsrs)] {
        pub const LEPTOS_OUTPUT_NAME: &str = "docsrs";
        pub const TALLYWEB_VERSION: &str = "0.3.6";
    } else {
        pub const LEPTOS_OUTPUT_NAME: &str = env!("LEPTOS_OUTPUT_NAME");
        pub const TALLYWEB_VERSION: &str = env!("TALLYWEB_VERSION");
    }
}

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

pub type SelectionSignal = leptos::RwSignal<components::SelectionModel<uuid::Uuid, Countable>>;
pub type StateResource =
    leptos::Resource<UserSession, Result<CountableStore, leptos::ServerFnError>>;

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
    #[error("Expired Token")]
    ExpiredToken,
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
    #[error("Could not find requested countable id")]
    CountableNotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Indexed db error: {0}")]
    Indexed(String),
    #[error("Javascript error: {0}")]
    Javascript(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Incorrect or missing env variable: {0}")]
    Environment(String),
    #[error("Countable Requires at least 1 leaf node")]
    RequiresChild,
    #[error("Encountered an invalid `string` while parsing `Color`")]
    InvalidColor,
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

impl From<indexed_db::Error<AppError>> for AppError {
    fn from(value: indexed_db::Error<AppError>) -> Self {
        match value {
            indexed_db::Error::NotInBrowser => Self::Indexed(value.to_string()),
            indexed_db::Error::IndexedDbDisabled => Self::Indexed(value.to_string()),
            indexed_db::Error::OperationNotSupported => Self::Indexed(value.to_string()),
            indexed_db::Error::OperationNotAllowed => Self::Indexed(value.to_string()),
            indexed_db::Error::InvalidKey => Self::Indexed(value.to_string()),
            indexed_db::Error::VersionMustNotBeZero => Self::Indexed(value.to_string()),
            indexed_db::Error::VersionTooOld => Self::Indexed(value.to_string()),
            indexed_db::Error::InvalidCall => Self::Indexed(value.to_string()),
            indexed_db::Error::InvalidArgument => Self::Indexed(value.to_string()),
            indexed_db::Error::AlreadyExists => Self::Indexed(value.to_string()),
            indexed_db::Error::DoesNotExist => Self::Indexed(value.to_string()),
            indexed_db::Error::DatabaseIsClosed => Self::Indexed(value.to_string()),
            indexed_db::Error::ObjectStoreWasRemoved => Self::Indexed(value.to_string()),
            indexed_db::Error::ReadOnly => Self::Indexed(value.to_string()),
            indexed_db::Error::FailedClone => Self::Indexed(value.to_string()),
            indexed_db::Error::InvalidRange => Self::Indexed(value.to_string()),
            indexed_db::Error::CursorCompleted => Self::Indexed(value.to_string()),
            indexed_db::Error::User(err) => err,
            _ => unreachable!(),
        }
    }
}

impl From<wasm_bindgen::JsValue> for AppError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::Javascript(value.as_string().unwrap_or_default())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value.to_string())
    }
}

pub fn connect_on_window_resize(f: Box<dyn FnMut()>) {
    let closure = Closure::wrap(f as Box<dyn FnMut()>);
    leptos_dom::window().set_onresize(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}
