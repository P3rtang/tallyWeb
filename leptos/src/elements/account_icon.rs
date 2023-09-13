use crate::app::{navigate, AccountAccentColor, SessionUser};
use leptos::*;

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
    let session_user = expect_context::<RwSignal<Option<SessionUser>>>(cx);
    let initial = create_read_slice(cx, session_user, move |user| {
        user.clone()
            .map(|u| {
                u.username
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string())
            })
            .flatten()
            .unwrap_or_default()
    });

    let show_overlay = create_rw_signal(cx, false);

    let colour = move || {
        format!(
            "background: #{}",
            letter_to_three_digit_hash(initial().chars().next().unwrap_or_default()),
        )
    };

    view! { cx,
        <Show
            when=move || { session_user().is_some() }
            fallback=|_| {view! { cx,  }}
        >
            <div id="user-icon" style=colour on:click=move |_| { show_overlay.update(|s| *s = !*s) }>
                <b>{ move || {
                    initial()
                }}</b>
            </div>
            <AccountOverlay show_overlay=show_overlay/>
        </Show>
    }
}

#[component]
pub fn AccountOverlay(cx: Scope, show_overlay: RwSignal<bool>) -> impl IntoView {
    let close_signal = expect_context::<Trigger>(cx);
    create_effect(cx, move |_| {
        close_signal.track();
        show_overlay.set(false);
    });

    let accent_color = expect_context::<Signal<AccountAccentColor>>(cx);
    let border_style = move || format!("border: 2px solid {}", accent_color.get());

    view! { cx ,
        <Show
            when=move || { show_overlay.get() }
            fallback=|_| { view! { cx,  } }
        >
            <div id="account-overlay" style=border_style>
                <LogoutButton/>
            </div>
        </Show>
    }
}

#[component]
pub fn LogoutButton(cx: Scope) -> impl IntoView {
    let logout = move |_| navigate(cx, "/login");

    view! { cx,
        <button class="overlay-button" on:click=logout>
            <i class="fa-solid fa-right-from-bracket"></i>
            <span>Logout</span>
        </button>
    }
}
