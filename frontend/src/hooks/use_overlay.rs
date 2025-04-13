#![allow(dead_code)]

use super::*;
use leptos::prelude::*;

pub fn use_overlay() -> Result<(impl Fn(ViewFn), impl Fn()), AppError> {
    let page_context = use_context::<PageContext>()
        .ok_or(AppError::PageContextUnavailable("use_overlay".to_string()))?;

    let overlay = StoredValue::new(page_context.overlay.clone());

    let set_body = {
        move |view: ViewFn| {
            overlay
                .get_value()
                .is_open
                .set(!overlay.get_value().is_open.get());
            overlay.get_value().body.set(view.clone());
        }
    };

    let close = {
        let page_context = page_context.clone();
        move || page_context.overlay.is_open.set(false)
    };

    Ok((set_body, close))
}
