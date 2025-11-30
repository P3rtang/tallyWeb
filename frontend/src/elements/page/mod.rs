use super::*;
use components::{Direction, ResizeBar};

stylance::import_style!(style, "./page.module.scss");

pub const SIDEBAR_MIN_WIDTH: usize = 280;

#[slot]
pub struct PageContent {
    #[prop(default = false.into(), into)]
    hide_border: Signal<bool>,

    #[prop(into)]
    attrs: AnyAttribute,

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

    #[prop(into, optional)]
    on_resize: EventCallback<usize>,

    children: ChildrenFn,
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
    let screen = hooks::use_screen();
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
            (has_transition.get() && screen.get().viewport() > ViewPort::Small)
                .then_some("transition-width")
        )
    };

    let handle_resize = move |ev: ev::DragEvent| {
        if ev.client_x() as usize > SIDEBAR_MIN_WIDTH {
            set_has_transition(false);
            if let Some(mut on_resize) = sidebar.get_value().map(|sb| sb.on_resize) {
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
                .is_some_and(|sb| sb.auto_hide.get())
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

    let width_style = move || {
        if screen.get().viewport() <= ViewPort::Small && sidebar_in_view() {
            return "100vw".to_string();
        };

        format!(
            "{}px",
            sidebar_in_view()
                .then_some(sidebar_width())
                .flatten()
                .unwrap_or_default()
        )
    };

    let accent = use_accent();

    view! {
        <div {..accent} class=page_classes style=css_vars>
            <Show when=has_sidebar>
                <div style:width=width_style class=sidebar_classes>
                    {(sidebar.get_value().unwrap().children)()}
                </div>
                <ResizeBar
                    direction=Direction::Vertical
                    position=Signal::derive(move || sidebar_width().unwrap_or_default())
                    on:drag=handle_resize
                />
            </Show>
            <div class=style::body>
                <Show when=has_navbar>{(navbar.get_value().unwrap().children)()}</Show>
                <div class=classes>
                    <div
                        style:border=move || {
                            if (page_content.hide_border)() { "none" } else { "" }
                        }
                        style:box-shadow=move || {
                            if (page_content.hide_border)() { "0px 0px 2px 0px black" } else { "" }
                        }
                    >
                        <div style:height="100%" {..page_content.attrs}>
                            {(page_content.children)()}
                        </div>
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
            .ok_or(super::AppError::InvalidColor(value.to_string()))
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let Self::RGB(r, g, b) = self;
        write!(f, "#{r:x}{g:x}{b:x}")
    }
}
