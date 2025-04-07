#![allow(unused)]
// TODO: add a hook to add breakpoints to a component

use super::*;

use components::{jar::MessageKey, MessageJar, NotificationConfig};
use elements::Button;
use indexed::IndexedSaveHandler;
use leptos::server_fn::error::ServerFnErrorErr;
use leptos_router::hooks::use_location;
use page_context::PageContext;
use std::future::Future;
use std::sync::Arc;
#[cfg(feature = "ssr")]
use tokio::task::spawn_blocking;

mod use_dialog;
mod use_history;
mod use_message;
mod use_overlay;
mod use_saving;
mod use_screen;

pub use use_dialog::use_confirm;
pub use use_history::{use_history, History};
pub use use_message::{use_message, Severity};
pub use use_overlay::use_overlay;
pub use use_saving::{use_local_saving, use_saving};
pub use use_screen::use_screen;
