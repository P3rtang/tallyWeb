#![allow(unused)]
// TODO: add a hook to add breakpoints to a component

use super::*;

use indexed::IndexedSaveHandler;
use leptos_router::hooks::use_location;
use page_context::PageContext;
#[cfg(feature = "ssr")]
use tokio::task::spawn_blocking;

mod use_overlay;
mod use_referer;
mod use_saving;
mod use_screen;

pub use use_overlay::use_overlay;
pub use use_referer::{use_referer, Options as RefererOptions};
pub use use_saving::{use_local_saving, use_saving};
pub use use_screen::use_screen;
