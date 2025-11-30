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
pub fn WithStoreAssign(store: CountableStore, children: ChildrenFn) -> impl IntoView {
    let store = RwSignal::<CountableStore>::new(store);
    provide_context(store);

    children()
}

#[component]
pub fn WithStore(children: ChildrenFn) -> impl IntoView {
    // TODO: add a loader/spinner as fallback

    let session = expect_context::<RwSignal<UserSession>>();

    let store_resource = Resource::new_blocking(session, move |user| async move {
        server::get_countable_store(user).await.ok()
    });

    provide_context(store_resource);

    // let _store_indexed_resource = LocalResource::new(move || async move {
    //     let save_handler = indexed::IndexedSaveHandler::new().await;
    //     if let Ok(s) = save_handler {
    //         s.get_store(session.get().user_uuid).await.ok()
    //     } else {
    //         None
    //     }
    // });
    // owner.with(|| provide_context(_store_indexed_resource));

    let children = StoredValue::new(children);

    view! {
        <Transition>
            {move || {
                store_resource
                    .get()
                    .flatten()
                    .map(move |store| {
                        view! { <WithStoreAssign store>{children.get_value()()}</WithStoreAssign> }
                    })
            }}
        </Transition>
    }
}
