use super::CountableStore;
use leptos::{server, ServerFnError};

#[server(GetCountableStore, "/api/session")]
pub async fn get_countable_store(user: uuid::Uuid) -> Result<CountableStore, ServerFnError> {
    use super::{super::api, Countable, CountableKind};
    use std::collections::{HashMap, VecDeque};

    let mut conn = api::extract_pool().await?.begin().await?;

    let mut store: HashMap<Countable, CountableKind> = HashMap::new();
    let mut counters: VecDeque<backend::DbCounter> =
        backend::counter::all_by_user(&mut conn, user).await?.into();
    let phases = backend::phase::all_by_user(&mut conn, user).await?;

    while let Some(c) = counters.pop_front() {
        // TODO: allow parent field on counters
        store.insert(c.uuid.into(), c.into());
    }

    for phase in phases {
        if let Some(parent) = store.get(&phase.parent_uuid.into()) {
            let uuid = phase.uuid;
            let countable: CountableKind = phase.into();
            parent.add_child(countable.clone())?;
            store.insert(uuid.into(), countable);
        }
    }

    conn.commit().await?;
    Ok(CountableStore { store })
}
