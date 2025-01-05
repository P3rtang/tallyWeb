use super::*;
use crate::UserSession;
// use components::MessageJar;
use leptos::prelude::*;

pub fn provide_store() -> (
    Resource<Option<CountableStore>>,
    LocalResource<Option<CountableStore>>,
) {
    // TODO: readd msg
    //
    // let msg = expect_context::<MessageJar>();

    let user = expect_context::<RwSignal<UserSession>>();

    let store_resource = Resource::new_blocking(user, move |user| async move {
        server::get_countable_store(user.user_uuid).await.ok()
    });

    let store_indexed_resource = LocalResource::new(move || async move {
        let save_handler = indexed::IndexedSaveHandler::new().await;
        if let Ok(s) = save_handler {
            s.get_store(user.get().user_uuid).await.ok()
        } else {
            None
        }
    });

    provide_context(store_resource);
    provide_context(store_indexed_resource);

    (store_resource, store_indexed_resource)
}
