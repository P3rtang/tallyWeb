use super::*;
use leptos::{create_effect, expect_context};

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

    pub async fn sync_store(&self, store: &mut CountableStore) -> Result<(), AppError> {
        let factory = indexed_db::Factory::get()?;
        let owner = store.owner();
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
        store.checked_mut().merge(local_store.checked())?;
        self.save(Box::new(store.clone()), Box::new(|_| ()))?;

        Ok(())
    }
}

impl SaveHandler for IndexedSaveHandler {
    fn save(
        &self,
        value: Box<dyn Savable>,
        on_error: Box<dyn Fn(&dyn std::error::Error) + 'static>,
    ) -> Result<(), AppError> {
        let msg = expect_context::<components::MessageJar>();

        #[allow(clippy::borrowed_box)]
        let action = leptos::create_action(move |value: &Box<dyn Savable>| {
            let value = value.clone_box();
            async move {
                let factory = indexed_db::Factory::<AppError>::get()?;
                let db = factory.open_latest_version("TallyWeb").await?;
                let store_name = value.indexed_db_name();

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
        create_effect(move |_| match action.value()() {
            Some(Err(err)) => {
                on_error(&err);
                msg.without_timeout().set_err(err)
            }
            _ => {}
        });

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn SaveHandler> {
        Box::new(self.clone())
    }
}
