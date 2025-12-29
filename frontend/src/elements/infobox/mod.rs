use super::*;

// modules
mod header;
mod infobox;

// imports
use app::UserName;
use chrono::Duration;
use components::Progressbar;
use home::Selection;
use hooks::{use_breakpoint, use_history};
use hooks::{use_local_saving_with_signal, use_saving};
use leptos::ev;
use leptos_router::hooks::use_params;
use web_sys::MouseEvent;

// internal
use header::InfoHeader;
use icon::*;
use menu::{Menu, MenuButton, MenuEntry};
use text::Text;

// re-exports
pub use infobox::InfoBox;

// stylance css classes
stylance::import_style!(style, "infobox.module.scss");
