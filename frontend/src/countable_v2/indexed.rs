use super::*;
use indexed_db;
use leptos::{create_effect, expect_context, Signal, WriteSignal};
use leptos_use::utils::JsonCodec;

#[allow(dead_code)]
#[derive(Clone)]
pub struct IndexedSaveHandler {
    version: u32,
    last_save: (
        Signal<Option<chrono::NaiveDateTime>>,
        WriteSignal<Option<chrono::NaiveDateTime>>,
    ),
}

impl IndexedSaveHandler {
    pub async fn new() -> Result<Self, AppError> {
        let last_sync = leptos_use::storage::use_local_storage::<
            Option<chrono::NaiveDateTime>,
            JsonCodec,
        >("indexed_last_sync");
        let version = 1;

        let factory = indexed_db::Factory::<AppError>::get()?;
        factory
            .open("TallyWeb", version, |evt| async move {
                let obj_builder = evt.database().build_object_store("Countable");
                obj_builder.create()?;
                Ok(())
            })
            .await?;

        Ok(Self {
            version,
            last_save: (last_sync.0, last_sync.1),
        })
    }
}

impl SaveHandler for IndexedSaveHandler {
    fn save(
        &self,
        value: Box<dyn Savable>,
        on_error: Box<dyn Fn(&dyn std::error::Error) + 'static>,
    ) -> Result<(), AppError> {
        let msg = expect_context::<components::MessageJar>();
        let set_ls = self.last_save.1;

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

        create_effect(move |_| match action.value()() {
            Some(Err(err)) => {
                on_error(&err);
                msg.without_timeout().set_err(err)
            }
            Some(_) => {
                set_ls(Some(chrono::Utc::now().naive_utc()));
            }
            _ => {}
        });

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn SaveHandler> {
        Box::new(self.clone())
    }
}
