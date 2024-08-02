use super::AppError;
use leptos::{create_action, create_effect, expect_context, Signal, WriteSignal};
use leptos_use::utils::JsonCodec;
use std::error::Error;

#[typetag::serde(tag = "type")]
pub trait Savable {
    fn indexed_db_name(&self) -> String;
    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + 'a>>;
    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), leptos::ServerFnError>>>>;
    fn message(&self) -> Option<leptos::View>;
    fn clone_box(&self) -> Box<dyn Savable>;
}

pub type ErrorFn = Box<dyn Fn(&dyn Error) + 'static>;

pub trait SaveHandler {
    fn save(&self, value: Box<dyn Savable>, on_error: ErrorFn) -> Result<(), AppError>;
    fn clone_box(&self) -> Box<dyn SaveHandler>;
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct ServerSaveHandler {
    last_save: (
        Signal<Option<chrono::NaiveDateTime>>,
        WriteSignal<Option<chrono::NaiveDateTime>>,
    ),
}

impl ServerSaveHandler {
    pub fn new() -> Self {
        let last_sync = leptos_use::storage::use_local_storage::<
            Option<chrono::NaiveDateTime>,
            JsonCodec,
        >("server_last_sync");
        Self {
            last_save: (last_sync.0, last_sync.1),
        }
    }
}

impl SaveHandler for ServerSaveHandler {
    fn save(
        &self,
        value: Box<dyn Savable>,
        on_error: Box<dyn Fn(&dyn Error) + 'static>,
    ) -> Result<(), AppError> {
        let msg = expect_context::<components::MessageJar>();
        let set_ls = self.last_save.1;

        #[allow(clippy::borrowed_box)]
        let action = create_action(move |val: &Box<dyn Savable>| val.save_endpoint());

        let msg_id = value
            .message()
            .map(|msg_view| msg.with_handle().set_msg_view(msg_view));
        action.dispatch(value);

        create_effect(move |_| {
            match action.value()() {
                Some(Err(err)) => {
                    if let Some(id) = msg_id {
                        msg.fade_out(id);
                    }
                    if !is_offline(&err) {
                        msg.without_timeout().set_server_err(&err);
                        on_error(&leptos::ServerFnErrorErr::from(err))
                    }
                }
                Some(_) => {
                    if let Some(id) = msg_id {
                        msg.fade_out(id);
                    }
                    set_ls(Some(chrono::Utc::now().naive_utc()));
                }
                _ => {}
            };
        });

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn SaveHandler> {
        Box::new(*self)
    }
}

fn is_offline(err: &leptos::ServerFnError) -> bool {
    matches!(err, leptos::ServerFnError::Request(_))
}

pub struct SaveHandlers {
    handlers: Vec<Box<dyn SaveHandler>>,
}

impl SaveHandlers {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn connect_handler(&mut self, handler: Box<dyn SaveHandler>) {
        self.handlers.push(handler)
    }
}

impl SaveHandler for SaveHandlers {
    fn save(
        &self,
        value: Box<dyn Savable>,
        on_error: Box<dyn Fn(&dyn Error) + 'static>,
    ) -> Result<(), AppError> {
        let res = || -> Result<(), AppError> {
            for h in self.handlers.iter() {
                h.save(value.clone_box(), Box::new(|_| ()))?
            }
            Ok(())
        };

        res().inspect_err(|err| {
            on_error(err);
        })?;

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn SaveHandler> {
        Box::new(self.clone())
    }
}

impl Clone for SaveHandlers {
    fn clone(&self) -> Self {
        Self {
            handlers: self.handlers.iter().map(|h| h.clone_box()).collect(),
        }
    }
}
