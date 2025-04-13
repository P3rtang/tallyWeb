use super::*;

// modules
mod header;
mod infobox;

// imports
use app::UserName;
use chrono::Duration;
use components::Progressbar;
use elements::icon::*;
use home::Selection;
use hooks::use_saving;
use hooks::{use_breakpoint, use_history};
use leptos::ev;
use leptos_router::hooks::use_params;
use web_sys::MouseEvent;

// internal
use header::InfoHeader;

// re-exports
pub use infobox::InfoBox;

// stylance css classes
stylance::import_style!(style, "infobox.module.scss");
