use super::*;

#[cfg(not(feature = "ssr"))]
use crate::hooks::use_saving;
use crate::EditWindow;
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
                <meta name="description" content="TallyWeb a website to keep track of shiny hunts in pokemon, it tracks the counter, the time and some more statistics about the hunt or phase, Author: P3rtang, icons: svgrepo.com and P3rtang" />
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

        <Link rel="icon" as_="image" type_="image/ico" href="/favicon.svg" />
        <link href="https://fonts.googleapis.com/css?family=Roboto" rel="stylesheet" media="print" onload=r#"this.media='all'"# />

        <Title text="TallyWeb" />

        <Router>
            <main on:click=close_overlay>
            {page_context}
                <Routes fallback=move || view!{<h1>Not Found</h1>}>
                    <Route path=path!("/public") view=move || View::new(()) />
                    <Route path=path!("/login") view=LoginPage/>
                    <ParentRoute path=path!("/") view=RouteUser ssr=leptos_router::SsrMode::Async>
                        <Route path=path!("preferences") view=PrefsWindow />
                        <Route path=path!("") view=Redirect />
                        <ParentRoute path=path!(":id") view=Outlet>
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
    let (store_rsc, _local_store_rsc) = provide_store();
    let pref_rsc = provide_prefs();

    let prefs = RwSignal::new(Preferences::default());
    provide_context(prefs);

    let store = RwSignal::new(CountableStore::default());
    provide_context(store);

    #[cfg(not(feature = "ssr"))]
    let saving = StoredValue::new(use_saving());

    let screen_rsc = Resource::new_blocking(|| (), async move |_| screen::server::get_screen().await.unwrap_or_default());
    provide_context(screen_rsc);

    view! {
        <Transition fallback=|| ()>
            { move || {
                user_rsc.track();
                pref_rsc.track();
                screen_rsc.track();

                if let Some(p) = pref_rsc.get() {
                    prefs.set(p)
                }

                #[cfg(feature="ssr")]
                {
                    if let Some(s) = store_rsc.get().flatten() {
                        store.set(s)
                    }
                }

                // INFO: This is done to enable full server side rendering because a local resource
                // would show the transition fallback, and then the client requires JS enabled
                #[cfg(not(feature="ssr"))]
                {
                    match (
                        store_rsc.get().flatten(),
                        _local_store_rsc.get().and_then(|s| s.take()),
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
                }
            }}
            <Outlet/>
        </Transition>
    }
}
