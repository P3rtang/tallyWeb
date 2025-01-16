use std::collections::HashSet;

use crate::EditWindow;

use super::{
    elements::*, page_context::PageContext, provide_prefs, provide_store, session::provide_session,
    CountableId, CountableStore, LoginPage, LEPTOS_OUTPUT_NAME,
};
use components::Separator;
use hooks::use_saving;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Meta, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes, A},
    hooks::use_query,
    params::{Params, ParamsError},
    path,
};
use serde::{Deserialize, Serialize};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let page_context = PageContext::new();
    provide_context(page_context.clone());

    let close_overlay = {
        let overlay = page_context.overlay.clone();
        move |_| overlay.is_open.set(false)
    };

    view! {
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Meta name="mobile-web-app-capable" content="yes" />

        <Stylesheet href=format!("/pkg/{LEPTOS_OUTPUT_NAME}.css") />
        <Stylesheet href="/fa/css/all.css" />

        <Link rel="icon" as_="image" type_="image/ico" href="/favicon.svg" />
        <Link href="https://fonts.googleapis.com/css?family=Roboto" rel="stylesheet" />

        <Title text="TallyWeb" />

        <Router>
            <main on:click=close_overlay>
            {page_context}
                <Routes fallback=move || view!{<h1>Not Found</h1>}>
                    <Route path=path!("/public") view=move || View::new(()) />
                    <Route path=path!("/login") view=LoginPage/>
                    <ParentRoute path=path!("/") view=Outlet ssr=leptos_router::SsrMode::Async>
                        <ParentRoute path=path!(":id") view=RouteUser>
                            <Route path=path!("") view=Body />
                            <Route path=path!("edit") view=EditWindow />
                        </ParentRoute>
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Params, Clone, Debug, PartialEq)]
pub struct UserName {
    pub id: String,
}

#[component]
pub fn RouteUser() -> impl IntoView {
    let user_rsc = provide_session();
    let (store_rsc, local_store_rsc) = provide_store();
    let pref_rsc = provide_prefs();

    let store = RwSignal::new(CountableStore::default());
    provide_context(store);

    let saving = StoredValue::new(use_saving());

    view! {
        <Transition fallback=|| ()>
            { move || {
                user_rsc.track();
                pref_rsc.track();

                match (
                    store_rsc.get().flatten(),
                    local_store_rsc.get().and_then(|s| s.take()),
                ) {
                    (Some(mut s), Some(l)) => {
                        s.merge(l);
                        saving.get_value()(s.clone());
                        store.set(s);
                    }
                    (Some(s), None) => store.set(s),
                    (None, Some(l)) => {
                        store.set(l);
                    },
                    (None, None) => {}
                }
            }}
            <Outlet/>
        </Transition>
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    slct: HashSet<CountableId>,
}

impl Selection {
    fn new() -> Self {
        Self {
            slct: HashSet::new(),
        }
    }

    pub fn contains(&self, key: &CountableId) -> bool {
        self.slct.contains(key)
    }
}

impl Params for Selection {
    fn from_map(map: &leptos_router::params::ParamsMap) -> std::result::Result<Self, ParamsError> {
        let selection = map
            .clone()
            .into_iter()
            .filter_map(|p| {
                p.0.starts_with("slct")
                    .then_some(p.0.to_string() + "=" + &p.1)
            })
            .collect::<Vec<_>>()
            .join("&");

        // strip off brackets
        serde_qs::from_str::<Self>(&selection)
            .map_err(|err| ParamsError::Params(std::sync::Arc::new(err)))
    }
}

impl IntoIterator for Selection {
    type Item = CountableId;

    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.slct.into_iter()
    }
}

#[component]
fn Body() -> impl IntoView {
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

#[component]
fn SidebarContent(#[prop(into)] width: Signal<usize>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let selection = expect_context::<Memo<Selection>>();
    let store_rsc = expect_context::<Resource<Option<CountableStore>>>();

    let each = move || {
        let mut root = store.get().root_node_ids();
        root.sort_by_key(|a| store.get().name(a));
        root
    };

    let width = move || format!("{}px", width.get());

    let is_selected = move |key: CountableId| selection.get().contains(&key);

    let action = ServerAction::<api::CreateCountable>::new();
    let on_submit = move |_| store_rsc.refetch();

    // TODO: when creating a counter automatically create a first phase as well
    Effect::new(move |_| {
        match action.value().get() {
            Some(Ok(countables)) => store.update(|s| countables.into_iter().for_each(|c| s.add_countable(c))),
            // TODO: add in logging of server error with messagejar
            Some(Err(_err)) => (),
            None => (),
        }
    });

    view! {
        <div style:width=width>
            <nav/>
            <List
                each
                key=|c| *c
                children=(move |c| store.get().children(&c)).into()
            >
                <RowSlot is_selected let:child slot>
                    <TreeRow countable=child />
                </RowSlot>
                <Separator slot><hr /></Separator>
            </List>
            <ActionForm action style:padding="0px 12px" on:submit=on_submit>
                <input type="hidden" name="kind" value=CountableKind::Counter.to_string() />
                <Button class:hover-darken=true style:width="100%" attr:r#type="submit">
                    <div>New Counter</div>
                </Button>
            </ActionForm>
        </div>
    }
}

#[component]
fn TreeRow(countable: CountableId) -> AnyView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let selection = expect_context::<Memo<Selection>>();

    let include_selection = move || {
        let mut sel = selection.get();
        if !sel.slct.remove(&countable) {
            sel.slct.insert(countable);
        }
        sel
    };

    let href = move || {
        format!(
            "?{}",
            serde_qs::to_string(&include_selection()).unwrap_or_default()
        )
    };

    view! { <A href style:width="100%">{store.get().name(&countable)}</A> }.into_any()
}
