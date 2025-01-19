use super::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let (show_sidebar, set_show_sidebar) = signal(true);
    let (width, set_width) = signal(400);

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
            <PageSidebar width is_shown=show_sidebar on_resize=set_width slot>
                <SidebarContent width/>
            </PageSidebar>
            <PageNavbar slot>
                <Navbar show_sidebar on_close_sidebar=set_show_sidebar/>
            </PageNavbar>
        </Page>
    }
}
