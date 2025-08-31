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
use hooks::{Severity, use_message};

use leptos::either::either;
use leptos_router::{components::*, hooks::*, params::*};
