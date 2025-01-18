#![allow(unused)]

pub(crate) use super::*;

use chrono::TimeDelta;
use components::*;
use leptos::{
    attr::{
        any_attribute::{AnyAttribute, IntoAnyAttribute},
        Attribute,
    },
    attribute_interceptor::AttributeInterceptor,
    ev, html,
    prelude::*,
};
use std::sync::Arc;

mod account;
mod button;
mod form;
mod icon;
mod infobox;
mod list;
mod navbar;
mod page;

pub use account::icon::AccountIcon;
pub use button::*;
pub use form::*;
pub use icon::*;
pub use infobox::InfoBox;
pub use list::{List, RowSlot, Separator};
pub use navbar::{Navbar, OnClose};
pub use page::{Color, OnResize, Page, PageContent, PageNavbar, PageSidebar};
