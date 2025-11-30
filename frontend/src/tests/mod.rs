use super::*;

// modules
mod notification;
mod page;

// re-exports
pub use page::TestPage;

// internal imports
use notification::TestNotifications;

// imports
use components::{Block, Slider, block};
use elements::{
    Button, Navbar, TextField,
    list::{List, RowSlot},
    page::*,
};
use hooks::{use_breakpoint, use_message};

use leptos::{attr::any_attribute::IntoAnyAttribute, either::either};
use leptos_router::{components::*, hooks::*, params::*};
