use super::{overlay::AccountOverlay, *};
use leptos::prelude::*;

stylance::import_style!(style, "./icon.module.scss");

pub fn letter_to_three_digit_hash(letter: char) -> String {
    use rand::{Rng, SeedableRng};

    let mut rng = rand::rngs::StdRng::seed_from_u64(letter as u64);
    let random_hash = rng.gen_range(0x6..=0xF);
    let random_hash2 = rng.gen_range(0x6..=0xF);
    let random_hash3 = rng.gen_range(0x6..=0xF);
    format!(
        "{:x}{:x}{:x}{:x}{:x}{:x}",
        random_hash, random_hash, random_hash2, random_hash2, random_hash3, random_hash3
    )
}

#[component]
pub fn AccountIcon<F>(username: F) -> impl IntoView
where
    F: Fn() -> String + Sync + Send + 'static,
{
    let initial = move || {
        username()
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default()
    };

    let (overlay, _) = hooks::use_overlay().unwrap();

    let open_overlay = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        overlay(
            (move || {
                view! {
                    <AccountOverlay />
                }
            })
            .into(),
        )
    };

    view! {
        <div
            data-testid="test-account-icon"
            class=style::icon
            on:click=open_overlay
        >
            <b>{move || { initial() }}</b>
        </div>
    }
}
