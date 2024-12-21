use components::{Direction, ResizeBar};
use leptos::*;

stylance::import_style!(style, "./style/page.module.scss");

pub const SIDEBAR_MIN_WIDTH: usize = 280;

pub type Sidebar = std::rc::Rc<dyn Fn(MaybeSignal<usize>) -> Fragment>;

pub type NavbarComponent = ChildComponent<()>;

pub trait ComponentState: Clone + 'static {}

#[derive(Clone)]
pub struct ChildComponent<T: Clone> {
    fragment: std::rc::Rc<dyn Fn(T) -> Fragment>,
    state: T,
}

impl<T: ComponentState> From<(Box<dyn Fn(T) -> Fragment>, T)> for ChildComponent<T> {
    fn from(value: (Box<dyn Fn(T) -> Fragment>, T)) -> Self {
        Self {
            fragment: std::rc::Rc::new(value.0),
            state: value.1,
        }
    }
}

impl From<Box<dyn Fn() -> Fragment>> for ChildComponent<()> {
    fn from(value: Box<dyn Fn() -> Fragment>) -> Self {
        Self {
            fragment: std::rc::Rc::new(move |_| value()),
            state: (),
        }
    }
}

impl From<(Box<dyn Fn(()) -> Fragment>, ())> for ChildComponent<()> {
    fn from(value: (Box<dyn Fn(()) -> Fragment>, ())) -> Self {
        Self {
            fragment: std::rc::Rc::new(value.0),
            state: (),
        }
    }
}

impl<T: ComponentState> IntoView for ChildComponent<T> {
    fn into_view(self) -> View {
        #[allow(unused_braces)]
        view! { {(self.fragment)(self.state)} }.into()
    }
}

impl IntoView for ChildComponent<()> {
    fn into_view(self) -> View {
        #[allow(unused_braces)]
        view! { {(self.fragment)(())} }.into()
    }
}

#[component]
pub fn Page(
    children: ChildrenFn,
    #[prop(optional, into)] navbar: Option<NavbarComponent>,
    #[prop(optional, into)] sidebar: Option<Sidebar>,
    #[prop(into, default=true.into())] show_sidebar: MaybeSignal<bool>,
    #[prop(optional, into, default=Color::default().into())] accent: MaybeSignal<Color>,
    #[prop(optional, into, default=false.into())] auto_hide_sidebar: MaybeSignal<bool>,
) -> impl IntoView {
    let (sidebar_width, set_sidebar_width) = create_signal(400);

    let navbar = store_value(navbar);
    let sidebar = store_value(sidebar);

    let has_navbar = move || navbar.get_value().is_some();
    let has_sidebar = move || sidebar.get_value().is_some();

    let content_class = move || {
        stylance::classes!(
            style::content,
            (!show_sidebar() || sidebar.get_value().is_none()).then_some(style::full_width)
        )
    };

    let (has_transition, set_has_transition) = create_signal(true);
    let sidebar_classes = move || {
        format!(
            "{} {}",
            style::sidebar,
            has_transition()
                .then_some("transition-width")
                .unwrap_or_default()
        )
    };

    let handle_resize = move |ev: ev::DragEvent| {
        if ev.client_x() as usize > SIDEBAR_MIN_WIDTH {
            set_has_transition(false);
            set_sidebar_width(ev.client_x() as usize);
        } else {
            set_has_transition(true);
        }
    };

    let css_vars = move || format!("--accent: {};", accent.get());

    let page_classes =
        move || stylance::classes!(style::page, auto_hide_sidebar().then_some(style::auto_hide));

    view! {
        <div class=page_classes style=css_vars>
            <Show when=has_sidebar>
                <div class=sidebar_classes>
                    {sidebar.get_value().unwrap()(sidebar_width.into())}
                </div>
            </Show>
            <ResizeBar direction=Direction::Vertical position=sidebar_width on:drag=handle_resize />
            <div class=style::body>
                <Show when=has_navbar>{navbar.get_value().unwrap()}</Show>
                <div class=content_class>
                    <div>{children}</div>
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
