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
    provide_context(store_resource);

    let store = create_rw_signal(CountableStore::default());
    provide_context(store);

    create_isomorphic_effect(move |_| match store_resource.get() {
        Some(Ok(s)) => {
            store.set(s);
        }
        Some(Err(err)) => {
            msg.set_err(err);
        }
        None => {}
    });

    view! {
        <Await future=move || server::get_countable_store(user.get_untracked().user_uuid) let:res>

            {
                if let Ok(data) = res {
                    store.set(data.clone());
                }
                children()
            }

        </Await>
    }
}
