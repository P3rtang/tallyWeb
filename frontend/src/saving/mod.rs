#![allow(unused)]
use super::AppError;
use leptos::prelude::*;
use std::error::Error;

mod server;
pub use server::ServerSaveHandler;

pub type ErrorFn = Box<dyn Fn(&dyn Error) + 'static>;

pub trait Savable: Send + Sync {
    fn has_change(&self) -> bool;
}

pub trait ServerSavable: Savable {
    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ServerFnError>> + Send + Sync>>;
}

pub trait LocalSavable: Savable {
    const INDEXED_DB_NAME: &str;

    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + 'a>>;
}

pub trait SaveHandler<S: Savable + 'static>: Send + Sync {
    fn save(&self, value: S, on_error: ErrorFn);
}
