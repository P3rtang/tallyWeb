use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarLayout {
    Portrait,
    Hover,
    Landscape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowSidebar(pub bool);

#[component(transparent)]
pub fn Sidebar(
    #[prop(optional, into, default=ShowSidebar(true).into())] display: MaybeSignal<ShowSidebar>,
    #[prop(optional, into, default=400.into())] width: MaybeSignal<usize>,
    #[prop(optional, into, default=SidebarLayout::Hover.into())] layout: MaybeSignal<SidebarLayout>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    children: ChildrenFn,
) -> impl IntoView {
    let aside_transform = move || match (layout(), display().0) {
        (SidebarLayout::Landscape, false) => {
            "transform: TranslateX(-2px); width: 0px; overflow-x: hidden;".into()
        }
        (SidebarLayout::Landscape, true) => {
            format!("width: {}px;", width())
        }
        (SidebarLayout::Portrait, true) => "width: 100vw;".into(),

        (_, false) => "transform: TranslateX(-120%);".into(),
        (_, true) => Default::default(),
    };

    let sidebar_style = move || match layout() {
        SidebarLayout::Landscape => format!("width: {}px", width()),
        SidebarLayout::Hover => format!("width: {}px", width() - 12),
        SidebarLayout::Portrait => String::new(),
    };

    view! {
        <aside {..attrs} style=aside_transform>
            <side-bar data-testid="test-sidebar" style=sidebar_style>
                {children()}
            </side-bar>
        </aside>
    }
}
