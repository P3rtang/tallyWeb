use super::*;

mod confirm;
mod dialog;

use std::{
    sync::Mutex,
    task::{Poll, Waker},
    thread,
    time::Duration,
};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    future::{ok, poll_fn},
    stream::StreamFuture,
    FutureExt, SinkExt, Stream, StreamExt,
};

use leptos::task::spawn_local;

stylance::import_style!(style, "./dialog.module.scss");

// re-exports
pub use confirm::use_confirm;
