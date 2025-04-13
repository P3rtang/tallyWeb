use super::*;
use crate::UserSession;
// use components::MessageJar;
use leptos::prelude::*;

pub fn provide_store() -> (
    Resource<Option<CountableStore>>,
    LocalResource<Option<CountableStore>>,
) {
    let session = expect_context::<Resource<UserSession>>();

    let store_resource = Resource::new_blocking(
        move || session.get(),
        move |user| async move {
            server::get_countable_store(user.unwrap_or_default())
                .await
                .ok()
        },
    );

    let store_indexed_resource = LocalResource::new(move || async move {
        let save_handler = indexed::IndexedSaveHandler::new().await;
        if let Ok(s) = save_handler {
            s.get_store(session.get().unwrap_or_default().user_uuid)
                .await
                .ok()
        } else {
            None
        }
    });

    provide_context(store_resource);
    provide_context(store_indexed_resource);

    (store_resource, store_indexed_resource)
}

#[component]
pub fn WithStore(owner: Owner, children: ChildrenFn) -> impl IntoView {
    // TODO: add a loader/spinner as fallback

    let session = expect_context::<RwSignal<UserSession>>();

    let store_resource = Resource::new_blocking(session, move |user| async move {
        server::get_countable_store(user).await.ok()
    });
    owner.with(|| provide_context(store_resource));

    let _store_indexed_resource = LocalResource::new(move || async move {
        let save_handler = indexed::IndexedSaveHandler::new().await;
        if let Ok(s) = save_handler {
            s.get_store(session.get().user_uuid).await.ok()
        } else {
            None
        }
    });
    owner.with(|| provide_context(_store_indexed_resource));

    let store = RwSignal::<CountableStore>::default();
    owner.with(|| provide_context(store));

    #[cfg(not(feature = "ssr"))]
    let saving = StoredValue::new(crate::hooks::use_saving());

    view! {
        <Transition>
            {move || {
                #[cfg(feature = "ssr")]
                {
                    if let Some(s) = store_resource.get().flatten() {
                        store.set(s)
                    }
                }
                #[cfg(not(feature = "ssr"))]
                {
                    match (
                        store_resource.get().flatten(),
                        _store_indexed_resource.get().and_then(|mut s| s.take()),
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
                        }
                        (None, None) => {}
                    }
                }
                children()
            }}
        </Transition>
    }
}
