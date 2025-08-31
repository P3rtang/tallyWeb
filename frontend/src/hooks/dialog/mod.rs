use super::*;

mod confirm;
#[allow(clippy::module_inception)]
mod dialog;

use std::{
    sync::Mutex,
    task::{Poll, Waker},
    thread,
    time::Duration,
};

use futures::{
    FutureExt, SinkExt, Stream, StreamExt,
    channel::mpsc::{Receiver, Sender, channel},
    future::{ok, poll_fn},
    stream::StreamFuture,
};

use leptos::ev::MouseEvent;
use leptos::task::spawn_local;

stylance::import_style!(style, "./dialog.module.scss");

// re-exports
pub use confirm::use_confirm;
