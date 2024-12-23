use super::*;
use components::{CloseOverlays, ToolTip};
use leptos::*;
use leptos_router::A;

pub type OnClose = std::rc::Rc<dyn Fn(bool)>;

impl FromClosure<bool> for OnClose {
    type Output = ();

    fn from_closure(closure: impl Fn(bool) -> Self::Output + 'static) -> Self {
        std::rc::Rc::new(closure)
    }
}

#[component]
pub fn Navbar(
    #[prop(default=true.into(), into)] has_sidebar: MaybeSignal<bool>,
    #[prop(default = false.into(), into)] show_sidebar: MaybeSignal<bool>,
    #[prop(optional)] on_close_sidebar: Option<OnClose>,
) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let close_overlay_signal = expect_context::<RwSignal<CloseOverlays>>();

    let accent_color = create_read_slice(preferences, |pref| pref.accent_color.0.clone());

    let on_close_sidebar = StoredValue::new(on_close_sidebar);

    let toggle_sidebar = move |_| {
        if let Some(f) = on_close_sidebar.get_value() {
            f(!show_sidebar())
        }
    };
    let close_overlays = move |_| close_overlay_signal.update(|_| ());

    let home_img_ref = create_node_ref::<html::Img>();

    view! {
        <nav on:click=close_overlays>
            <button
                id="toggle-sidebar"
                aria-label="toggle sidebar"
                on:click=toggle_sidebar
                disabled=move || !has_sidebar()
            >
                <img
                    height="32px"
                    width="32px"
                    style
                    src=move || {
                        if show_sidebar() {
                            "/icons/sidebar-left-closed-svgrepo-com-white.svg"
                        } else {
                            "/icons/sidebar-left-svgrepo-com-white.svg"
                        }
                    }
                />
            </button>
            <A href="/">
                <img
                    node_ref=home_img_ref
                    src="/favicon.svg"
                    width=48
                    height=48
                    alt="Home"
                    class="tooltip-parent"
                />
                <ToolTip parent_node=home_img_ref>Home</ToolTip>
            </A>

            <div style:margin-left="auto" style:display="flex" style:align-items="center">
                <StatusBar />
                <AccountIcon username=move || user.get().username accent_color />
            </div>
        </nav>
    }
}

#[component]
pub fn StatusBar() -> impl IntoView {
    view! {
        <Show when=|| false>
            <div id="status-bar">
                <Show when=|| false>
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
