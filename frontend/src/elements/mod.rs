#![allow(unused_variables)]
#![allow(clippy::module_inception)]
use super::*;

// modules
mod account;
pub mod button;
mod form;
pub mod icon;
mod infobox;
pub mod list;
pub mod menu;
mod message;
mod navbar;
pub mod page;
pub mod text;

// imports
use chrono::TimeDelta;
use components::*;
use hooks::use_accent;
use leptos::{
    attr::any_attribute::{AnyAttribute, IntoAnyAttribute},
    attribute_interceptor::AttributeInterceptor,
    either::{Either, EitherOf3},
    ev, html,
};
use leptos_router::components::A;
use std::sync::Arc;

// internal
use icon::*;

// re-exports
pub use account::icon::AccountIcon;
pub use button::*;
pub use form::*;
pub use fuzzy_sort::Sortable;
pub use infobox::InfoBox;
pub use message::Message;
pub use navbar::Navbar;
