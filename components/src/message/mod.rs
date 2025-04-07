use super::*;

// modules
mod injectable;
pub mod jar;
mod notification;

// re-exports
pub use injectable::{MessageProp, MessageSlot, ProvideMessageJar};
pub use notification::{NotificationConfig, NotificationKind};

// internal imports
use jar::{MessageJar as MJ, MessageKey};
use notification::Notification;

// imports
use chrono::Duration;
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use std::collections::HashMap;
