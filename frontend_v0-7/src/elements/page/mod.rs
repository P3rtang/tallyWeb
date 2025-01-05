use components::{Direction, FromEmptyClosure as FC, Prop, ResizeBar};
use leptos::{ev, prelude::*};

stylance::import_style!(style, "./page.module.scss");

pub const SIDEBAR_MIN_WIDTH: usize = 280;

#[slot]
pub struct PageContent {
    #[prop(default = false.into(), into)]
    hide_border: Signal<bool>,

    children: ChildrenFn,
}

#[derive(Clone)]
#[slot]
pub struct PageSidebar {
    #[prop(default = false.into(), into)]
    is_shown: Signal<bool>,

    #[prop(default = false.into(), into)]
    auto_hide: Signal<bool>,

    #[prop(default = 400.into(), into)]
    width: Signal<usize>,

    #[prop(optional)]
    on_resize: Option<OnResize>,

    children: ChildrenFn,
}

pub type OnResize = std::sync::Arc<dyn Fn(usize) + Send + Sync>;

pub trait FromClosure<T> {
    type Output;

    fn from_closure(closure: impl Fn(T) -> Self::Output + Send + Sync + 'static) -> Self;
}

impl FromClosure<usize> for OnResize {
    type Output = ();

    fn from_closure(closure: impl Fn(usize) + Send + Sync + 'static) -> Self {
        std::sync::Arc::new(closure)
    }
}

#[derive(Clone)]
#[slot]
pub struct PageNavbar {
    children: ChildrenFn,
}

#[component]
pub fn Page(
    page_content: PageContent,
    #[prop(optional)] page_sidebar: Option<PageSidebar>,
    #[prop(optional)] page_navbar: Option<PageNavbar>,
    #[prop(optional, into, default=Color::default().into())] accent: Signal<Color>,
) -> impl IntoView {
    let sidebar = StoredValue::new(page_sidebar);
    let navbar = StoredValue::new(page_navbar);

    let has_navbar = move || navbar.get_value().is_some();
    let has_sidebar = move || sidebar.get_value().is_some();

    let show_sidebar = move || sidebar.get_value().is_some_and(|sb| sb.is_shown.get());

    let sidebar_width = move || sidebar.get_value().map(|sb| sb.width.get());

    let (has_transition, set_has_transition) = signal(true);
    let sidebar_classes = move || {
        stylance::classes!(
            style::sidebar,
            has_transition.get().then_some("transition-width")
        )
    };

    let handle_resize = move |ev: ev::DragEvent| {
        if ev.client_x() as usize > SIDEBAR_MIN_WIDTH {
            set_has_transition(false);
            if let Some(on_resize) = sidebar.get_value().and_then(|sb| sb.on_resize) {
                on_resize(ev.client_x() as usize)
            }
        } else {
            set_has_transition(true);
        }
    };

    let css_vars = move || format!("--accent: {}", accent.get_untracked());

    let page_classes = move || {
        stylance::classes!(
            style::page,
            sidebar
                .get_value()
                .is_some_and(|sb| (sb.auto_hide)())
                .then_some(style::auto_hide)
        )
    };

    let sidebar_in_view = move || show_sidebar() && has_sidebar();

    let classes = move || {
        stylance::classes!(
            style::content,
            (!sidebar_in_view()).then_some(style::full_width)
        )
    };

    let position = StoredValue::new(Prop::<usize>::from_closure(move || {
        sidebar_width().unwrap_or(0)
    }));

    let width_style = move || {
        if sidebar_in_view() {
            format!("{}px", sidebar_width().unwrap_or(0))
        } else {
            "0px".to_string()
        }
    };

    view! {
        <div class=page_classes style=css_vars>
            <Show when=has_sidebar>
                <div style:width=width_style class=sidebar_classes>{(sidebar.get_value().unwrap().children)()}</div>
                <ResizeBar
                    direction=Direction::Vertical
                    position=position.get_value()
                    on:drag=handle_resize
                />
            </Show>
            <div class=style::body>
                <Show when=has_navbar>{(navbar.get_value().unwrap().children)()}</Show>
                <div class=classes>
                    <div>
                        <div style:border=move || {
                            if (page_content.hide_border)() { "none" } else { "" }
                        }>{(page_content.children)()}</div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    #[allow(clippy::upper_case_acronyms)]
    RGB(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Self {
        Self::RGB(139, 233, 253)
    }
}

impl TryFrom<&str> for Color {
    type Error = super::AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parse_string = move || {
            let r = u8::from_str_radix(&value[1..=2], 16).ok()?;
            let g = u8::from_str_radix(&value[3..=4], 16).ok()?;
            let b = u8::from_str_radix(&value[5..=6], 16).ok()?;

            Some(Self::RGB(r, g, b))
        };

        (value.starts_with('#') && value.len() == 7)
            .then(parse_string)
            .flatten()
            .ok_or(super::AppError::InvalidColor)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let Self::RGB(r, g, b) = self;
        write!(f, "#{r:x}{g:x}{b:x}")
    }
}
