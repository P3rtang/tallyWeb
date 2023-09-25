use leptos::*;
use wasm_bindgen::{prelude::Closure, JsCast};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenLayout {
    Small,
    Big,
}

impl ScreenLayout {
    fn get_position(&self) -> String {
        let position = match self {
            ScreenLayout::Small => "fixed",
            ScreenLayout::Big => "unset",
        };

        format!("position: {position};")
    }

    fn get_background(&self) -> String {
        let background = match self {
            ScreenLayout::Small => "#242424F8",
            ScreenLayout::Big => "none",
        };

        format!("background: {background};")
    }

    pub fn get_class(&self) -> &str {
        return match self {
            ScreenLayout::Small => "small",
            ScreenLayout::Big => "big",
        };
    }
}

#[derive(Debug, Clone)]
pub struct ShowSidebar(pub bool);

#[component]
pub fn Sidebar<F1, F2>(cx: Scope, display: F1, layout: F2, children: ChildrenFn) -> impl IntoView
where
    F1: Fn() -> ShowSidebar + 'static,
    F2: Fn() -> ScreenLayout + 'static,
{
    let sidebar_style = move || {
        layout().get_position()
            + if !display().0 {
                "transform: TranslateX(-25em); position: fixed;"
            } else {
                ""
            }
            + layout().get_background().as_str()
            + "height: 100%;"
            + "min-width: min(25rem, 100%);"
            + "transition: 0.35s;"
    };

    view! { cx,
        <div style=sidebar_style>
            { children(cx) }
        </div>
    }
}

pub fn connect_on_window_resize(f: Box<dyn FnMut()>) {
    let closure = Closure::wrap(f as Box<dyn FnMut() -> ()>);
    leptos_dom::window().set_onresize(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}
