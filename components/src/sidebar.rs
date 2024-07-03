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
    #[prop(optional, into)] width: Option<Signal<u32>>,
    layout: F2,
    #[prop(optional, default={ create_signal(String::from("#8BE9FD")).0.into() }, into)]
    accent_color: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView
where
    F1: Fn() -> ShowSidebar + Copy + 'static,
    F2: Fn() -> SidebarStyle + Copy + 'static,
{
    let w = move || width.map(|s| s.get()).unwrap_or(400);
    let aside_transform = move || match layout() {
        SidebarStyle::Landscape if !display().0 => {
            format!(
                "transform: TranslateX(-2px); width: {}px; overflow-x: hidden",
                0
            )
        }
        SidebarStyle::Landscape => {
            format!("border-right: 2px solid #FFFFFF80; width: {}px", w())
        }
        SidebarStyle::Hover if !display().0 => "transform: TranslateX(-120%);".to_string(),
        SidebarStyle::Portrait => "width: 100vw".to_string(),
        _ => Default::default(),
    };

    let sidebar_style = move || match layout() {
        SidebarStyle::Landscape => format!("width: {}px", w()),
        SidebarStyle::Hover => String::new(),
        SidebarStyle::Portrait => String::new(),
    };

    let aside_style = move || format!("--accent: {}; {}", accent_color(), aside_transform());

    view! {
        <aside style=aside_style>
            <side-bar data-testid="test-sidebar" style=sidebar_style>
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
