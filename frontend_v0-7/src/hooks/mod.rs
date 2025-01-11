#![allow(unused)]

use super::{
    indexed::IndexedSaveHandler, page_context::PageContext, AppError, LocalSavable, SaveHandler,
    ServerSavable, ServerSaveHandler,
};

use leptos::prelude::*;
use leptos_router::hooks::use_location;

mod use_overlay;
mod use_referer;
mod use_saving;

pub use use_overlay::use_overlay;
pub use use_referer::{use_referer, Options as RefererOptions};
pub use use_saving::use_saving;
