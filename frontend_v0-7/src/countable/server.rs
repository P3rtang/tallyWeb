use leptos::{server, server_fn::ServerFnError};

use super::*;

#[server(GetCountableStore, "/api/session")]
pub async fn get_countable_store(user: uuid::Uuid) -> Result<CountableStore, ServerFnError> {
    use super::{super::api, Countable, CountableId};
    use std::collections::{HashMap, VecDeque};

    let mut conn = api::extract_pool().await?.begin().await?;

    let mut store: HashMap<CountableId, Countable> = HashMap::new();
    let mut counters: VecDeque<backend::DbCounter> =
        backend::counter::all_by_user(&mut conn, user).await?.into();
    let phases = backend::phase::all_by_user(&mut conn, user).await?;

    while let Some(c) = counters.pop_front() {
        // TODO: allow parent field on counters
        store.insert(c.uuid.into(), c.into());
    }

    for phase in phases {
        store.insert(phase.uuid.into(), phase.into());
    }

    conn.commit().await?;

    Ok(CountableStore::new(user, store))
}
