use super::*;

stylance::import_style!(
    #[allow(dead_code)]
    style,
    "./icon.module.scss"
);

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
    Ellipsis,
    LogOut,
    LogIn,
    AddAccount,
    ArrowRight,
    ArrowLeft,
    ArrowUp,
    ArrowDown,
    Plus,
    HamburgerMenu,
    Visible,
    NotVisible,
    InscribedCheck,
    FilledInscribedCheck,
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
                IconKind::Ellipsis => style::ellipsis,
                IconKind::LogOut => style::log_out,
                IconKind::LogIn => style::log_in,
                IconKind::AddAccount => style::add_account,
                IconKind::ArrowRight => style::arrow_right,
                IconKind::ArrowLeft => style::arrow_left,
                IconKind::ArrowUp => style::arrow_up,
                IconKind::ArrowDown => style::arrow_down,
                IconKind::Plus => style::plus,
                IconKind::HamburgerMenu => style::hamburger,
                IconKind::Visible => style::visible,
                IconKind::NotVisible => style::not_visible,
                IconKind::InscribedCheck => style::inscribed_check,
                IconKind::FilledInscribedCheck => style::filled_inscribed_check,
            },
            style::icon
        )
    }
}

#[allow(dead_code)]
#[derive(Default, Clone, Copy)]
pub enum IconColor {
    #[default]
    White,
    Black,
    Accent,
    #[allow(clippy::upper_case_acronyms)]
    RGB(u8, u8, u8),
}

impl std::fmt::Display for IconColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconColor::White => write!(f, "white"),
            IconColor::Black => write!(f, "black"),
            IconColor::Accent => write!(f, "var(--accent)"),
            IconColor::RGB(r, g, b) => write!(f, "#{r:x}{g:x}{b:x}"),
        }
    }
}

#[component]
pub fn Icon(
    #[prop(into)] kind: Signal<IconKind>,
    #[prop(into, optional)] color: Signal<IconColor>,
    #[prop(optional)] xstyle: XStyle,
) -> impl IntoView {
    view! {
        <div
            class=move || kind.get().into_class()
            style:background=move || color.get().to_string()

            {..xstyle.into_any_attr()}
        />
    }
}
