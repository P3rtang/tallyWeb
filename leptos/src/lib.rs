#![feature(async_closure)]
#![feature(let_chains)]
#![feature(half_open_range_patterns_in_slices)]

pub mod app;

mod session;
pub(crate) use session::UserSession;
mod preferences;
pub(crate) use preferences::Preferences;

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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppError {
    #[error("Internal server Error")]
    Internal,
    #[error("Failed to write to browser LocalStorage")]
    SetLocalStorage,
    #[error("Connection Error")]
    Connection,
    #[error("Authentication Error")]
    Authentication,
    #[error("Failed to lock a Countable Mutex")]
    LockMutex,
    #[error("Error connecting to db pool\nGot Error: {0}")]
    DbConnection(String),
    #[error("User is missing auth_token")]
    MissingToken,
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
}

impl From<gloo_storage::errors::StorageError> for AppError {
    fn from(_: gloo_storage::errors::StorageError) -> Self {
        Self::SetLocalStorage
    }
}
