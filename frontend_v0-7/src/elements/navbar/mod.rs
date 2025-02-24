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
    let page_context = expect_context::<page_context::PageContext>();
    let session = expect_context::<RwSignal<UserSession>>();

    let home_img_ref = NodeRef::<html::Img>::new();

    let username = move || session.get().username;

    view! {
        <nav class=style::navbar>
            {page_context.sidebar.toggle_button()}
            <div class=style::icon>
                <A href=move || format!("/{}", username()) style:display="flex" style:align-items="center" attr:aria_label="home">
                    <Icon style:height="45px" style:width="45px" kind=IconKind::Favicon class:tooltip-parent=true />
                    <ToolTip parent_node=home_img_ref>Home</ToolTip>
                </A>
            </div>

            <div style:margin-left="auto" class=stylance::classes!(style::icon, style::round)>
                <AccountIcon username />
            </div>
        </nav>
    }
}
