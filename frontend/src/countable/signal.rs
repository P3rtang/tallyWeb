use super::super::UserSession;
use super::*;
use components::MessageJar;
use leptos::prelude::*;

pub async fn provide_store() -> Result<(), AppError> {
    let owner = Owner::current().unwrap();

    let user = expect_context::<RwSignal<UserSession>>();
    let msg = expect_context::<MessageJar>();

    let store_resource = Resource::new_blocking(user, move |user| async move {
        server::get_countable_store(user.user_uuid).await
    });
    owner.with(move || provide_context(store_resource));

    let store = RwSignal::new(CountableStore::default());
    owner.with(move || provide_context(store));

    Effect::new_isomorphic(move |_| match store_resource.get() {
        Some(Ok(s)) => {
            store.set(s);
        }
        Some(Err(err)) => {
            msg.set_err(err);
        }
        None => {}
    });

    store.set(store_resource.await?);

    Ok(())
}
