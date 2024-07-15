use leptos::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarStyle {
    Portrait,
    Hover,
    Landscape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
