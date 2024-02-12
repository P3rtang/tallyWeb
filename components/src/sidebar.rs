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

    fn get_navbar_style(&self) -> String {
        match self {
            SidebarStyle::Portrait => {
                "position: fixed; top: 52px;".to_string()
            },
            SidebarStyle::Hover => {
                "position: fixed; top: 52px; bottom: 12px; margin: 12px; height: calc(100vh - 80px); border-radius: 21px;".to_string()
            },
            SidebarStyle::Landscape => {
                "position: relative; border-right: 1px solid #FFFFFF80;".to_string()
            },
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
        layout().get_navbar_style()
            + "max-width: 25rem;"
            + "width: 100%;"
            + "transition: transform 0.5s, width 0.5s;"
            + "overflow-y: auto;"
            + if !display().0 && layout() != SidebarStyle::Hover {
                "width: 0px;"
            } else if !display().0 && layout() == SidebarStyle::Hover {
                "transform: TranslateX(-120%);"
            } else {
                ""
            }
            + &if layout() == SidebarStyle::Hover {
                format!("border: solid 2px {};", accent_color())
            } else {
                "".to_string()
            }
    };

    view! {
        <aside style=sidebar_style>
            <div class="content">{children()}</div>
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
