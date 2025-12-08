use leptos_reactive::{SignalGet, SignalSet};

use super::*;

pub fn use_saving<T: ServerSavable + LocalSavable + Clone + 'static>()
-> std::sync::Arc<dyn Fn(T) + Send + Sync + 'static> {
    let server_handler = ServerSaveHandler::new();

    #[cfg(feature = "hydrate")]
    let local_handler =
        AsyncDerived::new_unsync(async move || IndexedSaveHandler::new().await.ok());

    std::sync::Arc::new(move |item: T| {
        server_handler.save(item.clone(), Box::new(|_| {}));

        #[cfg(feature = "hydrate")]
        if let Some(local) = local_handler.get().flatten() {
            local.save(item, Box::new(|_| {}));
        }
    })
}

/**
   # returns
     - [Some]: When running on the client a callback is returned
     - [None]: When running on the server
     - [impl Fn(LocalSavable)]: The callback returned taking a Savable as parameter to save into indexedDB
*/
pub fn use_local_saving<T: LocalSavable + Clone + 'static>()
-> Option<std::sync::Arc<dyn Fn(T) + Send + Sync + 'static>> {
    #[cfg(feature = "ssr")]
    return None;

    let local_handler =
        AsyncDerived::new_unsync(async move || IndexedSaveHandler::new().await.ok());

    Some(std::sync::Arc::new(move |item: T| {
        if let Some(local) = local_handler.get().flatten() {
            local.save(item, Box::new(|_| {}));
        }
    }))
}

pub fn use_saving_v2<T, D>(data: D) -> impl Fn(T)
where
    D: Clone + 'static,
    T: OfflineSavableWithData<Data = D>,
    <<T as saving::OfflineSavableWithData>::Endpoint as leptos::server_fn::ServerFn>::Output: Clone,
    <<T as saving::OfflineSavableWithData>::Endpoint as leptos::server_fn::ServerFn>::Error: Clone,
    <T as saving::ToJsValue>::Error:
        std::convert::From<indexed_db::Error<<T as saving::ToJsValue>::Error>>,
{
    let action = RwSignal::<Option<ServerAction<T::Endpoint>>>::new(None);
    let input = RwSignal::<Option<T>>::new(None);

    Effect::new(move || {
        if let Some(action) = action.get() {
            match action.value().get() {
                Some(Ok(_)) => todo!(),
                Some(Err(_)) => {
                    // input.get().unwrap().diff();
                    if let Some(i) = input.get() {
                        save_object_store(i);
                    }
                }
                None => {}
            };
        }
    });

    return move |item: T| {
        input.set(Some(item.clone()));
        action.set(Some(item.save(None, data.clone())));
    };
}
