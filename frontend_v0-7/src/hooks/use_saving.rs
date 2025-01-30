use leptos::prelude::{AsyncDerived, Get};

use super::*;

pub fn use_saving<T: ServerSavable + LocalSavable + Clone + 'static>(
) -> std::sync::Arc<dyn Fn(T) + Send + Sync + 'static> {
    let server_handler = ServerSaveHandler::new();

    #[cfg(not(feature = "ssr"))]
    let local_handler =
        AsyncDerived::new_unsync(async move || IndexedSaveHandler::new().await.ok());

    std::sync::Arc::new(move |item: T| {
        server_handler.save(item.clone(), Box::new(|_| {}));

        #[cfg(not(feature = "ssr"))]
        if let Some(local) = local_handler.get().flatten() {
            local.save(item, Box::new(|_| {}));
        }
    })
}

/**
 * @returns
 *   - [None]: When running on the server
 *   - [Some]: When running on the client a callback is returned
 *   - [impl Fn(LocalSavable)]: The callback returned taking a Savable as parameter to save
 *   into indexedDB
 */
pub fn use_local_saving<T: LocalSavable + Clone + 'static>() -> Option<std::sync::Arc<dyn Fn(T) + Send + Sync + 'static>> {
    #[cfg(feature = "ssr")]
    return None;

    let local_handler = AsyncDerived::new_unsync(async move || IndexedSaveHandler::new().await.ok());

    Some(std::sync::Arc::new( move |item: T| {
        if let Some(local) = local_handler.get().flatten() {
            local.save(item, Box::new(|_| {}));
        }
    }))
}
