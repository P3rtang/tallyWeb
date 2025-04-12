#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(let_chains)]
#![allow(unused)]

mod attribute_fn;
pub mod block;
mod loading_screen;
mod message;
mod progressbar;
mod resizebar;
mod saving_screen;
mod select;
mod sidebar;
mod slider;
mod spinner;
mod time;
mod tooltip;
mod tree;
mod treeview;
mod types;
pub mod xstyle;

pub use attribute_fn::{AttributeFn, IntoAttributeFn};
pub use block::Block;
pub use loading_screen::*;
pub use message::*;
pub use progressbar::*;
pub use resizebar::{Direction, ResizeBar};
pub use saving_screen::*;
pub use select::{Select, SelectButton, SelectInput, SelectState};
pub use sidebar::*;
pub use slider::*;
pub use spinner::*;
pub use time::{Clock, Timer};
pub use tooltip::*;
pub use tree::{Caret, CaretState, ChildWrapper, RowWrapper, Separator, Tree, WrappedRowState};
pub use treeview::*;
pub use types::*;

pub type MessageJar = message::jar::MessageJar<message::jar::NoHandle>;

use leptos::{
    attr::{
        any_attribute::{AnyAttribute, IntoAnyAttribute},
        Attribute,
    },
    either::*,
    ev,
    logging::warn,
    prelude::*,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CloseOverlays();

#[component]
pub fn Overlay(
    show_overlay: RwSignal<bool>,
    location: ReadSignal<(i32, i32)>,
    children: ChildrenFn,
) -> impl IntoView {
    if let Some(close_signal) = use_context::<RwSignal<CloseOverlays>>() {
        Effect::new(move |_| {
            close_signal.track();
            show_overlay.set(false);
        });
    } else {
        warn!("No `close overlay` signal available");
    }

    let left = move || format!("{}px", location().0 + 10);
    let top = move || format!("{}px", location().1 + 10);

    view! {
        <Show when=move || { show_overlay.get() } fallback=|| ()>
            <div style:left=left style:top=top>
                {children()}
            </div>
        </Show>
    }
}
