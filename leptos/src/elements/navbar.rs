use super::*;
use components::{AccountIcon, CloseOverlays, ShowSidebar};
use leptos::*;
use leptos_router::A;

#[component]
pub fn Navbar() -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let accent_color = create_read_slice(preferences, |pref| pref.accent_color.0.clone());

    let show_sidebar = expect_context::<RwSignal<ShowSidebar>>();
    let toggle_sidebar = move |_| show_sidebar.update(|s| s.0 = !s.0);

    let close_overlay_signal = expect_context::<RwSignal<CloseOverlays>>();
    let close_overlays = move |_| {
        close_overlay_signal.update(|_| ());
    };

    view! {
        <nav on:click=close_overlays>
            <button id="toggle-sidebar" aria-label="toggle sidebar" on:click=toggle_sidebar>
                <i class="fa-solid fa-bars"></i>
            </button>
            <A href="/">
                <img src="/favicon.svg" width=48 height=48 alt="Home" class="tooltip-parent"/>
                <span class="tooltip bottom">Home</span>
            </A>

            <div style:margin-left="auto" style:display="flex" style:align-items="center">
                <StatusBar/>
                <AccountIcon username=move || user.get().username accent_color/>
            </div>
        </nav>
    }
}

#[component]
pub fn StatusBar() -> impl IntoView {
    let save_handler = expect_context::<super::SaveHandlerCountable>();

    let show_statusbar = move || save_handler.is_offline();

    view! {
        <Show when=show_statusbar>
            <div id="status-bar">
                <Show when=move || save_handler.is_offline()>
                    <svg
                        style:width="24px"
                        style:height="24px"
                        viewBox="0 0 48 48"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <title>wifi-disable</title>
                        <g id="Layer_2" data-name="Layer 2">
                            <g id="invisible_box" data-name="invisible box">
                                <rect width="48" height="48" fill="none"></rect>
                            </g>
                            <g id="Q3_icons" data-name="Q3 icons">
                                <g>
                                    <path
                                        fill="currentColor"
                                        d="M40.7,26.5a2,2,0,0,0-.2-2.6A23,23,0,0,0,24.3,17L29,21.7a18.6,18.6,0,0,1,8.7,5A1.9,1.9,0,0,0,40.7,26.5Z"
                                    ></path>
                                    <path
                                        fill="currentColor"
                                        d="M45.4,17.4A31.2,31.2,0,0,0,17.1,9.8l3.4,3.4L24,13a27.4,27.4,0,0,1,18.6,7.3,2,2,0,0,0,3-.2h0A2.1,2.1,0,0,0,45.4,17.4Z"
                                    ></path>
                                    <circle fill="currentColor" cx="24" cy="38" r="5"></circle>
                                    <path
                                        fill="currentColor"
                                        d="M5.4,3.6a1.9,1.9,0,0,0-2.8,0,1.9,1.9,0,0,0,0,2.8L9,12.8H8.7L6.8,14.1l-.3.3L4.7,15.7l-.3.2L2.6,17.4a2.1,2.1,0,0,0-.2,2.7h0a2,2,0,0,0,3,.2l1.7-1.4.4-.3,1.8-1.3.3-.2,2-1.1h0l.4-.2,3,3-.5.2-.6.3-.8.4-.6.3-.8.5-.5.4-.8.5-.5.4-.9.7-.4.3a11.4,11.4,0,0,1-1.1,1.1,2,2,0,0,0-.6,1.4,2.8,2.8,0,0,0,.4,1.2,1.9,1.9,0,0,0,3,.2l1.2-1,.3-.3.9-.7.4-.3,1.1-.7h.2l1.4-.8h.4l1.1-.5.5-.2h.3l3.3,3.3a16,16,0,0,0-9.1,5.3,1.9,1.9,0,0,0-.4,1.2,2,2,0,0,0,.4,1.3,2,2,0,0,0,3.1,0A11.5,11.5,0,0,1,24,29h1.2L38.6,42.4a1.9,1.9,0,0,0,2.8,0,1.9,1.9,0,0,0,0-2.8Z"
                                    ></path>
                                </g>
                            </g>
                        </g>
                    </svg>
                </Show>
            </div>
        </Show>
    }
}
