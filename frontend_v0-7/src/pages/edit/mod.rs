#![allow(unused)]
#![allow(clippy::module_inception)]
use super::*;

mod edit;

pub use edit::EditWindow;
use elements::{
    BoolField, Form, HeaderSlot, List, Navbar, Page, PageContent, PageNavbar, PageSidebar, RowSlot,
    SelectField, Separator, TextField, TimeDeltaField,
};

use crate::{
    app::UserName, hooks::use_referer, nodes::Masuda, session, session::SessionFormInput,
    CountableId, CountableStore, Hunttype, UserSession,
};
use leptos::{logging::*, prelude::*};
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params, use_query},
    params::Params,
};
