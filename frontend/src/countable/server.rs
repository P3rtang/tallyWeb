use super::*;

#[server(GetCountableStore, "/api/session_v2")]
pub async fn get_countable_store(session: UserSession) -> Result<CountableStore, ServerFnError> {
    use super::{super::api, Countable, CountableId};
    use std::collections::{HashMap, VecDeque};

    let mut conn = api::extract_pool().await?.begin().await?;

    let mut store: HashMap<CountableId, Countable> = HashMap::new();
    let mut counters: VecDeque<backend::DbCounter> =
        backend::counter::all_by_user(&mut conn, session.user_uuid)
            .await?
            .into();
    let phases = backend::phase::all_by_user(&mut conn, session.user_uuid).await?;

    while let Some(c) = counters.pop_front() {
        // TODO: allow parent field on counters
        store.insert(c.uuid.into(), c.into());
    }

    for phase in phases {
        store.insert(phase.uuid.into(), phase.into());
    }

    conn.commit().await?;

    Ok(CountableStore::new(session.user_uuid, store))
}
