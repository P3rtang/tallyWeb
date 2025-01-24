use super::*;

stylance::import_style!(style, "./icon.module.scss");

#[derive(Clone, Copy)]
pub enum IconKind {
    Cross,
    Edit,
    SidebarOpen,
    SidebarClosed,
    TrashCan,
    CaretRight,
    ArrowHeadDown,
    Favicon,
    Search,
    Sort,
    Arrow,
    Settings,
}

impl IconKind {
    fn into_class(self) -> String {
        stylance::classes!(
            match self {
                IconKind::Cross => style::cross,
                IconKind::Edit => style::edit,
                IconKind::SidebarOpen => style::sidebar_left_open,
                IconKind::SidebarClosed => style::sidebar_left_closed,
                IconKind::TrashCan => style::trash_can,
                IconKind::CaretRight => style::caret_right,
                IconKind::ArrowHeadDown => style::arrow_head_down,
                IconKind::Favicon => style::favicon,
                IconKind::Search => style::search,
                IconKind::Sort => style::sort,
                IconKind::Arrow => style::arrow,
                IconKind::Settings => style::settings,
            },
            style::icon
        )
    }
}

#[derive(Default, Clone, Copy)]
pub enum IconColor {
    #[default]
    White,
    Black,
    #[allow(clippy::upper_case_acronyms)]
    RGB(u8, u8, u8),
}

impl std::fmt::Display for IconColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconColor::White => write!(f, "white"),
            IconColor::Black => write!(f, "black"),
            IconColor::RGB(r, g, b) => write!(f, "#{r:x}{g:x}{b:x}"),
        }
    }
}

#[component]
pub fn Icon(
    #[prop(into)] kind: Signal<IconKind>,
    #[prop(into, optional)] color: Signal<IconColor>,
) -> impl IntoView {
    view! {
        <div
            class=move || kind.get().into_class()
            style:background=move || color.get().to_string()
        />
    }
}
