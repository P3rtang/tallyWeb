#![allow(non_snake_case)]

use super::*;
use components::{CloseOverlays, SidebarStyle};
use leptos::{logging::debug_warn, *};
use leptos_router::A;

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
pub fn AccountIcon<F>(
    username: F,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView
where
    F: Fn() -> String + 'static,
{
    let initial = move || {
        username()
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default()
    };

    let button_style = move || {
        accent_color
            .map(|ac| format!("background: {};", ac()))
            .unwrap_or_default()
    };

    let show_overlay = create_rw_signal(false);
    let open_overlay = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        show_overlay.update(|s| *s = !*s);
    };

    view! {
        <div
            id="user-icon"
            data-testid="test-account-icon"
            style=button_style
            on:click=open_overlay
        >
            <b>{move || { initial() }}</b>
        </div>
        <Show
            when=move || accent_color.is_some()
            fallback=move || view! { <AccountOverlay show_overlay/> }
        >
            <AccountOverlay show_overlay accent_color=accent_color.unwrap()/>
        </Show>
    }
}

#[component]
pub fn AccountOverlay(
    show_overlay: RwSignal<bool>,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView {
    if let Some(close_signal) = use_context::<RwSignal<CloseOverlays>>() {
        create_effect(move |_| {
            close_signal.track();
            show_overlay.set(false);
        });
    } else {
        debug_warn!("No `close overlay` signal available");
    }

    let screen_layout = expect_context::<RwSignal<SidebarStyle>>();

    let border_style = move || {
        accent_color
            .map(|ac| format!("border: 2px solid {};", ac()))
            .unwrap_or_default()
    };

    let show_about = create_rw_signal(false);

    view! {
        <Show when=show_overlay fallback=|| ()>
            <div
                id="account-overlay"
                data-testid="test-account-overlay"
                style=border_style
                on:click=move |ev: web_sys::MouseEvent| { ev.stop_propagation() }
            >
                <AccountOverlayNavigate
                    link="/preferences"
                    fa_icon="fa-solid fa-gear"
                    text="preferences"
                />
                <AccountOverlayButton
                    on_click=move || show_about.set(true)
                    fa_icon="fa-solid fa-circle-info"
                    text="about"
                />
                <hr/>
                <AccountOverlayNavigate
                    link="/login"
                    fa_icon="fa-solid fa-right-from-bracket"
                    text="Logout"
                />
            // TODO: remove session cookie
            </div>
        </Show>

        <Show
            when=move || accent_color.is_some()
            fallback=move || view! { <AboutDialog open=show_about layout=screen_layout/> }
        >
            <AboutDialog open=show_about layout=screen_layout accent_color=accent_color.unwrap()/>
        </Show>
    }
}

#[component]
pub fn AccountOverlayButton<F>(
    on_click: F,
    #[prop(default = true)] close_overlay: bool,
    #[prop(optional)] icon: Option<&'static str>,
    #[prop(optional)] fa_icon: Option<&'static str>,
    #[prop(optional)] text: Option<&'static str>,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <button
            class="overlay-button"
            on:click=move |_| {
                if close_overlay {
                    if let Some(t) = use_context::<RwSignal<CloseOverlays>>() {
                        t.update(|_| ())
                    }
                    on_click()
                }
            }
        >

            <Show when=move || fa_icon.is_some() fallback=|| ()>
                <i class=fa_icon.unwrap()></i>
            </Show>
            <Show when=move || icon.is_some() fallback=|| ()>
                <svg src=icon.unwrap()></svg>
            </Show>
            <Show when=move || text.is_some() fallback=|| ()>
                <span>{text.unwrap()}</span>
            </Show>
        </button>
    }
}

#[component]
pub fn AccountOverlayNavigate(
    link: &'static str,
    #[prop(default = true)] close_overlay: bool,
    #[prop(default = false)] show_link: bool,
    #[prop(optional)] icon: Option<&'static str>,
    #[prop(optional)] fa_icon: Option<&'static str>,
    #[prop(optional)] text: Option<&'static str>,
) -> impl IntoView {
    view! {
        <A href=link class=if !show_link { "remove-underline" } else { "" }>
            <button
                class="overlay-button"
                on:click=move |_| {
                    if close_overlay {
                        if let Some(t) = use_context::<RwSignal<CloseOverlays>>() {
                            t.update(|_| ())
                        }
                    }
                }
            >

                <Show when=move || fa_icon.is_some() fallback=|| ()>
                    <i class=fa_icon.unwrap()></i>
                </Show>
                <Show when=move || icon.is_some() fallback=|| ()>
                    <svg src=icon.unwrap()></svg>
                </Show>
                <Show when=move || text.is_some() fallback=|| ()>
                    <span>{text.unwrap()}</span>
                </Show>
            </button>
        </A>
    }
}
