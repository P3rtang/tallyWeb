use super::*;

pub mod account;
mod edit;
pub mod home;
mod login;
mod preferences;

pub use edit::EditWindow;
pub use login::LoginPage;
pub use preferences::PrefsWindow;

use components::EventCallback;
use fuzzy_sort::Sortable;
use leptos::{ev, html};
use leptos_router::{
    components::A,
    hooks::use_query,
    params::{Params, ParamsError},
};

use hooks::*;

use serde::{Deserialize, Serialize};

use elements::{
    BoolField, ColorField, Form, HeaderSlot, Navbar, SelectField, TextField, TimeDeltaField,
    button::*, icon::*, list::*, page::*,
};
