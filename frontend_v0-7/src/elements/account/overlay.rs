use super::hoc;
use components::CloseOverlays;
use leptos::prelude::*;
use leptos_router::components::A;

stylance::import_style!(style, "./overlay.module.scss");

#[component]
pub fn AccountOverlay() -> impl IntoView {
    let show_about = RwSignal::new(false);

    hoc::with_accent(view! {
        <div
            class=style::overlay
            id="account-overlay"
            data-testid="test-account-overlay"
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
            <hr />
            <AccountOverlayNavigate
                link="/login"
                fa_icon="fa-solid fa-right-from-bracket"
                text="Logout"
            />
        // TODO: remove session cookie
        </div>

        // <Show
        //     when=move || accent_color.is_some()
        //     fallback=move || view! { <AboutDialog open=show_about /> }
        // >
        //     <AboutDialog open=show_about accent_color=accent_color.unwrap() />
        // </Show>
    })
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
    let handle_click = move |_| {
        if close_overlay {
            if let Some(t) = use_context::<RwSignal<CloseOverlays>>() {
                t.update(|_| ())
            }
            on_click()
        }
    };

    view! {
        <button class=style::row on:click=handle_click>

            <div>
                <Show when=move || fa_icon.is_some() fallback=|| ()>
                    <i class=fa_icon.unwrap()></i>
                </Show>
                <Show when=move || icon.is_some() fallback=|| ()>
                    <svg src=icon.unwrap()></svg>
                </Show>
                <Show when=move || text.is_some() fallback=|| ()>
                    <span>{text.unwrap()}</span>
                </Show>
            </div>
        </button>
    }
}

#[component]
pub fn AccountOverlayNavigate(
    link: &'static str,
    #[prop(default = true)] close_overlay: bool,
    #[prop(optional)] icon: Option<&'static str>,
    #[prop(optional)] fa_icon: Option<&'static str>,
    #[prop(optional)] text: Option<&'static str>,
) -> impl IntoView {
    view! {
        <A href=link attr:class=stylance::classes!(style::row, "no-underline")>
            <div
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
            </div>
        </A>
    }
}
