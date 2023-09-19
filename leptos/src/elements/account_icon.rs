#![allow(non_snake_case)]

use crate::app::SessionUser;
use leptos::*;
use leptos_router::A;

use super::*;

pub fn letter_to_three_digit_hash(letter: char) -> String {
    use rand::{Rng, SeedableRng};

    let mut rng = rand::rngs::StdRng::seed_from_u64(letter as u64);
    let random_hash = rng.gen_range(0x6..=0xF);
    let random_hash2 = rng.gen_range(0x6..=0xF);
    let random_hash3 = rng.gen_range(0x6..=0xF);
    format!("{:x}{:x}{:x}", random_hash, random_hash2, random_hash3)
}

#[component]
pub fn AccountIcon(cx: Scope) -> impl IntoView {
    let user = expect_context::<Memo<Option<SessionUser>>>(cx);

    let initial = move || {
        user()
            .clone()
            .map(|u| {
                u.username
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string())
            })
            .flatten()
            .unwrap_or_default()
    };

    let preferences = expect_context::<RwSignal<crate::app::Preferences>>(cx);
    let style = create_read_slice(cx, preferences, |pref| {
        format!("background-color: {};", pref.accent_color.0)
    });

    let show_overlay = create_rw_signal(cx, false);
    let open_overlay = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        show_overlay.update(|s| *s = !*s);
    };

    view! { cx,
        <Show
            when=move || { user().is_some() }
            fallback=|_| {view! { cx,  }}
        >
            <div id="user-icon" style=style on:click=open_overlay>
                <b>{ move || { initial() }}</b>
            </div>
            <AccountOverlay show_overlay=show_overlay/>
        </Show>
    }
}

#[component]
pub fn AccountOverlay(cx: Scope, show_overlay: RwSignal<bool>) -> impl IntoView {
    if let Some(close_signal) = use_context::<RwSignal<CloseOverlays>>(cx) {
        create_effect(cx, move |_| {
            close_signal.get();
            show_overlay.set(false);
        });
    } else {
        debug_warn!("No `close overlay` signal available");
    }

    let preferences = expect_context::<RwSignal<crate::app::Preferences>>(cx);
    let border_style = move || format!("border: 2px solid {}", preferences.get().accent_color);

    view! { cx ,
        <Show
            when=move || { show_overlay.get() }
            fallback=|_| { view! { cx,  } }
        >
            <div id="account-overlay" style=border_style on:click=move |ev: web_sys::MouseEvent| { ev.stop_propagation() }>
                <AccountOverlayNavigate
                    link="/preferences"
                    fa_icon="fa-solid fa-gear"
                    text="preferences"
                />
                <hr/>
                <AccountOverlayNavigate
                    link="/login"
                    fa_icon="fa-solid fa-right-from-bracket"
                    text="Logout"
                />
            </div>
        </Show>
    }
}

#[component]
pub fn AccountOverlayButton(
    cx: Scope,
    #[prop(default = true)] close_overlay: bool,
    #[prop(optional)] icon: Option<&'static str>,
    #[prop(optional)] fa_icon: Option<&'static str>,
    #[prop(optional)] text: Option<&'static str>,
) -> impl IntoView {
    view! { cx,
        <button
            class="overlay-button"
            on:click=move |_| { if close_overlay {
                use_context::<RwSignal<CloseOverlays>>(cx).map(|t| t.update(|_| ()));
            }}
        >
            <Show
                when=move || fa_icon.is_some()
                fallback=|_| ()
            >
                <i class={ fa_icon.unwrap() }></i>
            </Show>
            <Show
                when=move || icon.is_some()
                fallback=|_| ()
            >
                <svg src={ icon.unwrap() }></svg>
            </Show>
            <Show
                when=move || text.is_some()
                fallback=|_| ()
            >
                <span>{ text.unwrap() }</span>
            </Show>
        </button>
    }
}

#[component]
pub fn AccountOverlayNavigate(
    cx: Scope,
    link: &'static str,
    #[prop(default = true)] close_overlay: bool,
    #[prop(default = false)] show_link: bool,
    #[prop(optional)] icon: Option<&'static str>,
    #[prop(optional)] fa_icon: Option<&'static str>,
    #[prop(optional)] text: Option<&'static str>,
) -> impl IntoView {
    view! { cx,
        <A
            href=link
            class={ if !show_link { "remove-underline" } else { "" } }
        >
            <button
                class="overlay-button"
                on:click=move |_| { if close_overlay {
                    use_context::<RwSignal<CloseOverlays>>(cx).map(|t| t.update(|_| ()));
                }}>
                <Show
                    when=move || fa_icon.is_some()
                    fallback=|_| ()
                >
                    <i class={ fa_icon.unwrap() }></i>
                </Show>
                <Show
                    when=move || icon.is_some()
                    fallback=|_| ()
                >
                    <svg src={ icon.unwrap() }></svg>
                </Show>
                <Show
                    when=move || text.is_some()
                    fallback=|_| ()
                >
                    <span>{ text.unwrap() }</span>
                </Show>
            </button>
        </A>
    }
}
