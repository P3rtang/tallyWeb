use super::*;

#[component]
pub fn TestPage() -> impl IntoView {
    let page_context = expect_context::<page_context::PageContext>();
    let sidebar_width = page_context.sidebar.width();
    let set_width = move |w| page_context.sidebar.set_width().set(w);

    let params = use_query::<SearchParams>();
    let topic: Memo<Topic> = Memo::new(move |_| params.get().unwrap_or_default().topic.into());

    view! {
        <Page>
            <PageContent hide_border=true slot>
                {
                    move || either!(topic.get(),
                        Topic::Notifications => view!{<TestNotifications />},
                        Topic::None => ().into_view(),
                    )
                }
            </PageContent>
            <PageSidebar is_shown=page_context.sidebar.is_shown() width=sidebar_width on_resize=set_width slot>
                <nav/>
                <Sidebar topic />
            </PageSidebar>
            <PageNavbar slot>
                <Navbar />
            </PageNavbar>
        </Page>
    }
}

#[component]
pub fn Sidebar(#[prop(into)] topic: Signal<Topic>) -> impl IntoView {
    let is_selected = move |t: &str| topic.get() == t;

    view! {
        <List
            each=|| ["Notifications"]
            key=|key| *key
        >
            <RowSlot is_selected let:topic slot>
                <TreeRow topic/>
            </RowSlot>
        </List>
    }
}

#[component]
pub fn TreeRow(#[prop(into)] topic: String) -> impl IntoView {
    view! {
        <A style:width="100%" href=format!("?topic={}", topic.to_lowercase())>{topic}</A>
    }
}

#[derive(Debug, Clone, Default, Params, PartialEq)]
pub struct SearchParams {
    topic: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Notifications,
    #[default]
    None,
}

impl From<Option<String>> for Topic {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(x) if x == "Notifications" => Self::Notifications,
            Some(x) if x == "notifications" => Self::Notifications,
            _ => Self::None,
        }
    }
}

impl PartialEq<&str> for Topic {
    fn eq(&self, other: &&str) -> bool {
        match (self, other.to_lowercase().as_str()) {
            (Self::Notifications, "notifications") => true,
            _ => false,
        }
    }
}
