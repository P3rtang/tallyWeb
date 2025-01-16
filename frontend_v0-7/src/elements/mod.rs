#![allow(unused)]

pub(crate) use super::*;

use chrono::TimeDelta;
use components::*;
use leptos::{
    attr::{
        any_attribute::{AnyAttribute, IntoAnyAttribute},
        Attribute, NextAttribute,
    },
    attribute_interceptor::AttributeInterceptor,
    ev, html,
    prelude::*,
};
use std::sync::Arc;

mod account;
mod form;
mod infobox;
mod list;
mod navbar;
mod page;
mod button;

pub use account::icon::AccountIcon;
pub use form::*;
pub use infobox::InfoBox;
pub use list::{List, RowSlot, Separator};
pub use navbar::{Navbar, OnClose};
pub use page::{Color, OnResize, Page, PageContent, PageNavbar, PageSidebar};
pub use button::*;
