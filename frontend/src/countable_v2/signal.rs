use super::super::UserSession;
use super::*;
use components::MessageJar;
use leptos::*;

#[component(transparent)]
pub fn ProvideStore(children: ChildrenFn) -> impl IntoView {
    let user = expect_context::<RwSignal<UserSession>>();
    let msg = expect_context::<MessageJar>();

    let store_resource = create_blocking_resource(user, move |user| async move {
        server::get_countable_store(user.user_uuid).await
    });

    let store = create_rw_signal(CountableStore::default());
    provide_context(store);

    view! {
        <Transition>

            {
                match store_resource.get() {
                    Some(Ok(s)) => store.set(s),
                    Some(Err(err)) => msg.set_err(err),
                    None => {}
                }
                children()
            }

        </Transition>
    }
}
