#![allow(unused)]
pub(crate) mod indexed;
pub(crate) mod nodes;
pub(crate) mod server;
mod signal;
pub(crate) mod store;

use super::*;

// modules

// imports
use leptos::server_fn::ServerFnError;

// internal

// re-exports
pub use nodes::{Countable, CountableId, CountableKind, Counter, Hunttype};
pub(crate) use store::CountableStore as CS;
pub type CountableStore = CS<store::Level, store::UnChecked>;
pub type StoreResource = Resource<Option<CountableStore>>;
pub use signal::{WithStore, provide_store};
