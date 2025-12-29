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

pub fn use_saving_signal<T: ServerSavable + LocalSavable + Clone + 'static>(
    initial: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    let saving = use_saving();

    let signal = signal(initial);

    Effect::new(move || {
        let item = signal.0.get();
        saving(item);
    });

    return signal;
}

pub fn use_local_saving_signal<T: LocalSavable + Clone + 'static>(
    initial: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    use_local_saving_rw_signal(initial).split()
}

pub fn use_local_saving_rw_signal<T: LocalSavable + Clone + 'static>(initial: T) -> RwSignal<T> {
    let signal = RwSignal::new(initial);

    #[cfg(feature = "ssr")]
    return signal;

    let saving = use_local_saving().unwrap();

    Effect::new(move || {
        let item = signal.get();
        saving(item);
    });

    return signal;
}

pub fn use_local_saving_with_signal<T, F>(getter: F)
where
    T: LocalSavable + Clone + 'static,
    F: Fn() -> T + 'static,
{
    #[cfg(feature = "ssr")]
    return;

    let saving = use_local_saving::<T>().unwrap();

    Effect::new(move || {
        let item = getter();
        saving(item);
    });
}
