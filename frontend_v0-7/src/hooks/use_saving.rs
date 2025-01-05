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
