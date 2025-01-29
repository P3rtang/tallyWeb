use super::*;
use components::ToolTip;
use leptos::{html, prelude::*};
use leptos_meta::Link;
use leptos_router::components::A;

stylance::import_style!(style, "./navbar.module.scss");

#[derive(Clone)]
pub struct OnClose(Arc<dyn Fn(bool) + Send + Sync + 'static>);

impl Default for OnClose {
    fn default() -> Self {
        Self(Arc::new(|_| ()))
    }
}

impl<F> From<F> for OnClose
where
    F: Fn(bool) + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

impl std::ops::FnOnce<(bool,)> for OnClose {
    type Output = ();

    extern "rust-call" fn call_once(self, args: (bool,)) -> Self::Output {
        (self.0)(args.0)
    }
}

#[component]
pub fn Navbar(
    #[prop(default=true.into(), into)] has_sidebar: Signal<bool>,
    #[prop(default = false.into(), into)] show_sidebar: Signal<bool>,
    #[prop(into, optional)] on_close_sidebar: OnClose,
) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();

    let on_close_sidebar = StoredValue::new(on_close_sidebar);

    let toggle_sidebar = move |_| on_close_sidebar.get_value()(!show_sidebar());

    let home_img_ref = NodeRef::<html::Img>::new();

    let icon = Signal::derive(move || {
        if show_sidebar.get() {
            IconKind::SidebarClosed
        } else {
            IconKind::SidebarOpen
        }
    });

    view! {
        <nav class=style::navbar>
            <button
                class=stylance::classes!("hover-lighten", "icon")
                aria-label="toggle sidebar"
                on:click=toggle_sidebar
                disabled=move || !has_sidebar()
            >
                <div class=stylance::classes!(style::sidebar_toggle, style::icon)>
                    <Icon kind=icon />
                </div>
            </button>
            <div class=style::icon>
                <A href=move || format!("/{}", user.get().username) style:display="flex" style:align-items="center" attr:aria_label="home">
                    <Icon style:height="45px" style:width="45px" kind=IconKind::Favicon class:tooltip-parent=true />
                    <ToolTip parent_node=home_img_ref>Home</ToolTip>
                </A>
            </div>

            <div style:margin-left="auto" class=stylance::classes!(style::icon, style::round)>
                <AccountIcon username=move || user.get().username />
            </div>
        </nav>
    }
}
