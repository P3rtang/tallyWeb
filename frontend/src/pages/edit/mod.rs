#![allow(unused)]
#![allow(clippy::module_inception)]
use super::*;

// modules
mod edit;

// imports
use crate::{
    CountableId, CountableStore, Hunttype, UserSession, app::UserName, hooks::use_history,
    nodes::Masuda, session, session::SessionFormInput,
};
use leptos::{logging::*, prelude::*};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params, use_query},
    params::Params,
};

// internal

// re-exports
pub use edit::EditWindow;

// stylance css classes
stylance::import_style!(style, "./edit.module.scss");
