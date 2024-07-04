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
pub fn Sidebar(
    #[prop(optional, into, default=ShowSidebar(true).into())] display: MaybeSignal<ShowSidebar>,
    #[prop(optional, into)] width: Option<Signal<u32>>,
    #[prop(optional, into, default=SidebarStyle::Hover.into())] layout: MaybeSignal<SidebarStyle>,
    children: ChildrenFn,
) -> impl IntoView {
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
        _ if !display().0 => "transform: TranslateX(-120%);".to_string(),
        SidebarStyle::Portrait => "width: 100vw".to_string(),
        _ => Default::default(),
    };

    let sidebar_style = move || match layout() {
        SidebarStyle::Landscape => format!("width: {}px", w()),
        SidebarStyle::Hover => String::new(),
        SidebarStyle::Portrait => String::new(),
    };

    view! {
        <aside style=aside_transform>
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
