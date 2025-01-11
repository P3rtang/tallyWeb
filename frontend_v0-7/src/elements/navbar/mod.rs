use super::*;
use components::ToolTip;
use leptos::{html, prelude::*};
use leptos_meta::Link;
use leptos_router::components::A;

stylance::import_style!(style, "./navbar.module.scss");

pub type OnClose = std::sync::Arc<dyn Fn(bool) + Send + Sync>;

impl FromClosure<bool> for OnClose {
    type Output = ();

    fn from_closure(closure: impl Fn(bool) -> Self::Output + Send + Sync + 'static) -> Self {
        std::sync::Arc::new(closure)
    }
}

#[component]
pub fn Navbar(
    #[prop(default=true.into(), into)] has_sidebar: Signal<bool>,
    #[prop(default = false.into(), into)] show_sidebar: Signal<bool>,
    #[prop(optional)] on_close_sidebar: Option<OnClose>,
) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();

    let on_close_sidebar = StoredValue::new(on_close_sidebar);

    let toggle_sidebar = move |_| {
        if let Some(f) = on_close_sidebar.get_value() {
            f(!show_sidebar())
        }
    };

    let home_img_ref = NodeRef::<html::Img>::new();

    view! {
        <Link rel="preload" as_="image" type_="image/svg+xml" href="/icons/sidebar-left-svgrepo-com-white.svg" fetchpriority="high" />
        <Link rel="preload" as_="image" type_="image/svg+xml" href="/icons/sidebar-left-closed-svgrepo-com-white.svg"  fetchpriority="high"/>
        <nav class=style::navbar>
            <button
                class=stylance::classes!("hover-lighten")
                aria-label="toggle sidebar"
                on:click=toggle_sidebar
                disabled=move || !has_sidebar()
            >
                <div class=stylance::classes!(style::sidebar_toggle, style::icon)>
                    <img
                        height="32px"
                        width="32px"
                        class:hidden=move || !show_sidebar()
                        src="/icons/sidebar-left-closed-svgrepo-com-white.svg"
                    />
                    <img
                        height="32px"
                        width="32px"
                        class:hidden=show_sidebar
                        src="/icons/sidebar-left-svgrepo-com-white.svg"
                    />
                </div>
            </button>
            <div class=style::icon>
                <A href=move || format!("/{}", user.get().username)>
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
            </div>

            <div style:margin-left="auto" class=stylance::classes!(style::icon, style::round)>
                <AccountIcon username=move || user.get().username />
            </div>
        </nav>
    }
}
