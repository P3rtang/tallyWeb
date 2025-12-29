use super::*;

use crate::EditWindow;
use components::{MessageSlot, ProvideMessageJar};
use leptos_meta::{Link, Meta, MetaTags, Stylesheet, Title, provide_meta_context};
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
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta
                    name="description"
                    content="TallyWeb a website to keep track of shiny hunts in pokemon, it tracks the counter, the time and some more statistics about the hunt or phase, Author: P3rtang, icons: svgrepo.com and P3rtang"
                />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let page_context = page_context::PageContext::new();
    provide_context(page_context.clone());

    #[cfg(feature = "hydrate")]
    leptos::task::spawn_local(async {
        let _ = indexed::IndexedSaveHandler::new().await;
    });

    let owner = Owner::current().unwrap();

    let close_overlay = {
        let overlay = page_context.overlay.clone();
        move |_| overlay.is_open.set(false)
    };

    view! {
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Meta name="mobile-web-app-capable" content="yes" />

        <Stylesheet href=format!("/pkg/{LEPTOS_OUTPUT_NAME}.css") />

        <Link rel="icon" as_="image" type_="image/ico" href="/favicon.svg" />
        <link
            href="https://fonts.googleapis.com/css?family=Roboto"
            rel="stylesheet"
            media="print"
            onload=r#"this.media='all'"#
        />

        <Title text="TallyWeb" />

        <Router>
            <ProvideMessageJar owner>
                <MessageSlot slot let:state>
                    <elements::Message state />
                </MessageSlot>
            </ProvideMessageJar>
            <main on:click=close_overlay>
                {page_context} <Routes fallback=move || view! { <h1>Not Found</h1> }>
                    <Route path=path!("/public") view=move || View::new(()) />
                    <Route path=path!("/login") view=LoginPage />
                    <Route path=path!("/create-account") view=account::CreateAccount />
                    <Route path=path!("/terms") view=move || () />
                    <ParentRoute path=path!("/") view=RouteUser ssr=leptos_router::SsrMode::Async>
                        <Route path=path!("test") view=tests::TestPage />
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
    let session_rsc = session::provide_session();
    provide_context(session_rsc);

    #[cfg(not(feature = "ssr"))]
    let navigate = StoredValue::new(leptos_router::hooks::use_navigate());

    view! {
        <Transition fallback=|| ()>
            {move || {
                session_rsc.track();
                #[cfg(not(feature = "ssr"))]
                Effect::new(move |_| {
                    let user = session_rsc.get();
                    if let Some(user) = user {
                        navigate.get_value()(&format!("/{}", user.username), Default::default());
                    } else {
                        navigate.get_value()("/login", Default::default());
                    }
                });
            }}
        </Transition>
    }
}

#[derive(Params, Clone, Debug, PartialEq)]
pub struct UserName {
    pub id: String,
}

#[component]
pub fn RouteUser() -> impl IntoView {
    let session_rsc = session::provide_session();
    provide_context(session_rsc);

    let session = RwSignal::<UserSession>::default();
    provide_context(session);

    let screen_signal = RwSignal::<Screen>::default();
    provide_context(screen_signal);

    let prefs_rsc = provide_prefs(session);

    let pref_signal = RwSignal::<Preferences>::default();
    provide_context(pref_signal);

    let screen_rsc = Resource::new_blocking(
        || (),
        async move |_| screen::server::get_screen().await.unwrap_or_default(),
    );

    let session_signal = expect_context::<RwSignal<UserSession>>();

    let store_resource = Resource::new_blocking(session_signal, move |user| async move {
        server::get_countable_store(user.clone()).await.ok()
    });

    provide_context(store_resource);

    let store_signal = Signal::derive(move || store_resource.get().flatten().unwrap_or_default());
    provide_context(store_signal);

    let store_setter: SignalSetter<CountableStore> =
        (move |v: CountableStore| store_resource.set(Some(Some(v)))).into_signal_setter();

    provide_context(store_setter);

    let store_signal = RwSignal::<CountableStore>::default();
    provide_context(store_signal);

    view! {
        <Transition>
            {move || {
                if let Some(s) = session_rsc.get() {
                    session_signal.set(s)
                }
                if let Some(s) = screen_rsc.get() {
                    screen_signal.set(s)
                }
                if let Some(p) = prefs_rsc.get() {
                    pref_signal.set(p)
                }
                if let Some(s) = store_resource.get().flatten() {
                    store_signal.set(s);
                }

                view! { <Outlet /> }
            }}
        </Transition>
    }
}
