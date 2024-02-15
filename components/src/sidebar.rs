use leptos::*;
use wasm_bindgen::{prelude::Closure, JsCast};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarStyle {
    Portrait,
    Hover,
    Landscape,
}

impl SidebarStyle {
    pub fn get_widget_class(&self) -> &str {
        match self {
            SidebarStyle::Portrait => "small",
            SidebarStyle::Hover => "small",
            SidebarStyle::Landscape => "big",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShowSidebar(pub bool);

#[component(transparent)]
pub fn Sidebar<F1, F2>(
    display: F1,
    layout: F2,
    #[prop(optional, default={ create_signal(String::from("#8BE9FD")).0.into() }, into)]
    accent_color: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView
where
    F1: Fn() -> ShowSidebar + 'static,
    F2: Fn() -> SidebarStyle + 'static,
{
    let sidebar_style = move || {
        if !display().0 && layout() != SidebarStyle::Hover {
            "width: 0px; transform: TranslateX(-2px);"
        } else if !display().0 && layout() == SidebarStyle::Hover {
            "transform: TranslateX(-120%);"
        } else {
            ""
        }
    };

    view! {
        <aside>
            <side-bar
                data-testid="test-sidebar"
                style=move || format!("--accent: {}; {}", accent_color(), sidebar_style())
            >
                {children()}
            </side-bar>
        </aside>
    }
}

pub fn connect_on_window_resize(f: Box<dyn FnMut()>) {
    let closure = Closure::wrap(f as Box<dyn FnMut()>);
    leptos_dom::window().set_onresize(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}

#[component(transparent)]
pub fn HoverSidebar(children: ChildrenFn) -> impl IntoView {
    view! {
        <aside>
            <div class="content">{children()}</div>
        </aside>
    }
}
