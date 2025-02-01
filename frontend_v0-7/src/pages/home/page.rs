use super::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let page_context = expect_context::<page_context::PageContext>();
    let sidebar_width = page_context.sidebar.width();
    let set_width = move |w| page_context.sidebar.set_width().set(w);

    let params = use_query::<Selection>();
    let selection = Memo::new(move |_| params.get().ok().unwrap_or(Selection::new()));
    provide_context(selection);

    let countable_list = Signal::derive(move || {
        let mut sel = selection.get().into_iter().collect::<Vec<_>>();
        sel.sort_by_key(|a| store.get().name(a));
        sel
    });

    view! {
        <Page>
            <PageContent hide_border=true slot>
                <InfoBox countable_list />
            </PageContent>
            <PageSidebar is_shown=page_context.sidebar.is_shown() width=sidebar_width on_resize=set_width slot>
                <SidebarContent/>
            </PageSidebar>
            <PageNavbar slot>
                <Navbar />
            </PageNavbar>
        </Page>
    }
}
