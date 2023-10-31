#![feature(async_closure)]
#![feature(let_chains)]
#![feature(half_open_range_patterns_in_slices)]

pub mod app;
mod countable;
mod elements;
mod pages;
mod saving;

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

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Internal server Error")]
    Internal,
    #[error("Failed to write to browser LocalStorage")]
    SetLocalStorage(#[from] gloo_storage::errors::StorageError),
    #[error("Failed to lock a Countable Mutex")]
    LockMutex,
}
