#![allow(dead_code)]

use super::*;
use leptos::prelude::Effect;

#[cfg(not(docsrs))]
const IDB_VERSION: &str = env!("IDB_TALLYWEB_VERSION");
#[cfg(docsrs)]
const IDB_VERSION: &str = "1";

#[allow(dead_code)]
#[derive(Clone)]
pub struct IndexedSaveHandler {
    version: u32,
}

impl IndexedSaveHandler {
    pub async fn new() -> Result<Self, AppError> {
        let version = IDB_VERSION
            .parse()
            .map_err(|_| AppError::Environment("IDB_VERSION".to_string()))?;

        let factory = indexed_db::Factory::<AppError>::get()?;
        factory
            .open("TallyWeb", version, |evt| async move {
                let _ = evt.database().delete_object_store("Countable");
                let obj_builder = evt.database().build_object_store("Countable");
                obj_builder.create()?;
                Ok(())
            })
            .await?;

        Ok(Self { version })
    }

    #[allow(dead_code)]
    pub async fn reset() -> Result<(), AppError> {
        let factory = indexed_db::Factory::<AppError>::get()?;
        let db = factory.open_latest_version("TallyWeb").await?;
        db.transaction(&["Countable"])
            .rw()
            .run(|transaction| async move {
                transaction.object_store("Countable")?.clear().await?;
                Ok(())
            })
            .await?;

        Ok(())
    }

    pub async fn get_store(&self, owner: uuid::Uuid) -> Result<CountableStore, AppError> {
        let factory = indexed_db::Factory::get()?;
        let db = factory.open_latest_version("TallyWeb").await?;
        let map = db
            .transaction(&["Countable"])
            .run(move |evt| async move {
                let obj = evt.object_store("Countable")?;
                let map = obj
                    .get_all(None)
                    .await?
                    .into_iter()
                    .map(Countable::from_js)
                    .collect::<Result<Vec<Countable>, AppError>>()?
                    .into_iter()
                    .map(|c| (c.uuid().into(), c))
                    .collect::<std::collections::HashMap<CountableId, Countable>>();
                Ok(map)
            })
            .await?;

        let local_store = CountableStore::new(owner, map);

        Ok(local_store)
    }
}

impl<S: LocalSavable + Clone + 'static> SaveHandler<S> for IndexedSaveHandler {
    fn save(&self, value: S, on_error: ErrorFn) {
        #[allow(clippy::borrowed_box)]
        let action = leptos::prelude::Action::new_local(move |value: &S| {
            let value = value.clone();
            async move {
                let store_name = value.indexed_db_name();
                let factory = indexed_db::Factory::<AppError>::get()?;
                let db = factory.open_latest_version("TallyWeb").await?;

                let value = value.clone();
                db.transaction(&[store_name.as_str()])
                    .rw()
                    .run(move |tr| {
                        let obj = tr.object_store(&store_name);
                        async move {
                            value.save_indexed(obj?).await?;
                            Ok(())
                        }
                    })
                    .await?;

                Ok::<(), AppError>(())
            }
        });

        action.dispatch(value);

        #[allow(clippy::single_match)]
        Effect::new(move |_| match action.value().get() {
            Some(Err(err)) => {
                on_error(&err);
            }
            _ => {}
        });
    }
}
