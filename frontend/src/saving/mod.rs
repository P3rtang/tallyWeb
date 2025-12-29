#![allow(unused)]
use super::*;
use leptos::prelude::*;

mod server;

// re-export
pub use server::ServerSaveHandler;

pub type ErrorFn = Box<dyn Fn(&dyn std::error::Error) + 'static>;

pub trait Savable: Send + Sync {
    fn has_change(&self) -> bool;
}

pub trait ServerSavable: Savable {
    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ServerFnError>> + Send + Sync>>;
}

pub trait LocalSavable: Savable + serde::Serialize {
    const INDEXED_DB_NAME: &str;

    fn as_js(&self) -> AppResult<wasm_bindgen::JsValue> {
        Ok(js_sys::JSON::parse(&serde_json::to_string(self)?)?)
    }

    fn idb_key(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + 'a>> {
        let key = wasm_bindgen::JsValue::from_str(&self.idb_key());
        let value = self.as_js();
        Box::pin(async move {
            obj.put_kv(&key, &value?).await?;
            Ok(())
        })
    }
}

pub trait SaveHandler<S: Savable + 'static>: Send + Sync {
    fn save(&self, value: S, on_error: ErrorFn);
}
