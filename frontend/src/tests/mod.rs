use super::*;

// modules
mod notification;
mod page;

// re-exports
pub use page::TestPage;

// internal imports
use notification::TestNotifications;

// imports
use components::{block, Block, Slider};
use elements::{
    list::{List, RowSlot},
    page::*,
    Button, Navbar, TextField,
};
use hooks::{use_message, Severity};

use leptos::either::either;
use leptos_router::{components::*, hooks::*, params::*};
