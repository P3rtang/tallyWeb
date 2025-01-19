use super::*;

use crate::hooks::use_saving;
use crate::EditWindow;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Meta, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    params::Params,
    path,
};

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

    let page_context = page_context::PageContext::new();
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
                        <Route path=path!("") view=Redirect />
                        <ParentRoute path=path!(":id") view=RouteUser>
                            <Route path=path!("") view=crate::home::HomePage />
                            <Route path=path!("edit") view=EditWindow />
                        </ParentRoute>
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
pub fn Redirect() -> impl IntoView {
    let user_rsc = session::provide_session();

    #[cfg(not(feature = "ssr"))]
    let navigate = leptos_router::hooks::use_navigate();

    view! {
        <Transition fallback=|| ()>
        {
            user_rsc.track();
            #[cfg(not(feature = "ssr"))]
            Effect::new(move |_| {
                let user = user_rsc.get();
                if let Some(user) = user {
                    navigate(&format!("/{}", user.username), Default::default());
                } else {
                    // TODO: have a landing page
                    navigate("/login", Default::default());
                }
            });
        }
        </Transition>
    }
}

#[derive(Params, Clone, Debug, PartialEq)]
pub struct UserName {
    pub id: String,
}

#[component]
pub fn RouteUser() -> impl IntoView {
    let user_rsc = session::provide_session();
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
                        let has_change = s.merge(l);
                        if has_change {
                            saving.get_value()(s.clone());
                        }
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
