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
pub fn WithStoreAssign<Chil>(
    store: CountableStore,
    children: TypedChildrenFn<Chil>,
) -> impl IntoView
where
    Chil: IntoView + Send + 'static,
{
    let store = RwSignal::<CountableStore>::new(store);
    provide_context(store);

    children.into_inner()()
}

#[component]
pub fn WithStore<Chil>(children: TypedChildrenFn<Chil>) -> impl IntoView
where
    Chil: IntoView + Send + 'static,
{
    // TODO: add a loader/spinner as fallback

    // let _store_indexed_resource = LocalResource::new(move || async move {
    //     let save_handler = indexed::IndexedSaveHandler::new().await;
    //     if let Ok(s) = save_handler {
    //         s.get_store(session.get().user_uuid).await.ok()
    //     } else {
    //         None
    //     }
    // });
    // owner.with(|| provide_context(_store_indexed_resource));

    // Effect::new_sync(move || {
    //     store_resource.get().flatten().map(move |s| {
    //         store.set(s);
    //     });
    // });

    children.into_inner()()
}
