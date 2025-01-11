#![allow(dead_code)]
#![allow(unused_imports)]

use components::*;
use leptos::{attribute_interceptor::AttributeInterceptor, prelude::*};

mod account;
mod form;
mod infobox;
mod list;
mod navbar;
mod page;

pub(crate) use super::{app::UserName, countable, hoc, hooks, AppError, UserSession};

pub use account::icon::AccountIcon;
pub use form::form::Form;
pub use infobox::InfoBox;
pub use list::{List, RowSlot};
pub use navbar::{Navbar, OnClose};
pub use page::{Color, FromClosure, OnResize, Page, PageContent, PageNavbar, PageSidebar};
