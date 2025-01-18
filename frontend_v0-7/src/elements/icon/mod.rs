use super::*;

stylance::import_style!(style, "./icon.module.scss");

#[derive(Clone, Copy)]
pub enum IconKind {
    Cross,
    Edit,
    SidebarOpen,
    SidebarClosed,
}

impl IconKind {
    fn into_class(self) -> String {
        stylance::classes!(
            match self {
                IconKind::Cross => style::cross,
                IconKind::Edit => style::edit,
                IconKind::SidebarOpen => style::sidebar_left_open,
                IconKind::SidebarClosed => style::sidebar_left_closed,
            },
            style::icon
        )
    }
}

#[component]
pub fn Icon(#[prop(into)] kind: Signal<IconKind>) -> impl IntoView {
    view! {
        <div class=move || kind.get().into_class() />
    }
}
