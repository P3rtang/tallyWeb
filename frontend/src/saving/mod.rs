#![allow(unused)]
use super::*;
use leptos::{prelude::*, server_fn::ServerFn};
use std::error::Error;

mod server;
pub use server::ServerSaveHandler;

pub type ErrorFn = Box<dyn Fn(&dyn Error) + 'static>;

pub trait Savable: Send + Sync {
    fn has_change(&self) -> bool {
        return true;
    }
}

pub trait ServerSavable: Savable {
    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ServerFnError>> + Send + Sync>>;
}

pub trait LocalSavable: Savable {
    fn indexed_db_name(&self) -> String;
    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + 'a>>;
}

pub trait SaveHandler<S: Savable + 'static>: Send + Sync {
    fn save(&self, value: S, on_error: ErrorFn);
}

#[derive(Clone)]
pub enum SaveAction<T: OfflineSavable> {
    Diff(ServerAction<T::DiffEndpoint>),
    Full(ServerAction<T::Endpoint>),
}

pub trait OfflineSavable: Savable + Clone + Sized + 'static
where
    <<Self as OfflineSavable>::DiffEndpoint as ServerFn>::Output: Clone + std::marker::Sync,
    <<Self as OfflineSavable>::Endpoint as ServerFn>::Output: Clone + std::marker::Sync,
{
    type SaveDiff: OfflineSavable = ();
    type DiffEndpoint: ServerFn + Clone + Sync;
    type Endpoint: ServerFn + Clone + Sync;
    const OBJECT_STORE: &'static str = "saving";

    fn full(&self) -> Self::Endpoint;
    fn diff(&self, other: Self) -> Self::DiffEndpoint;

    fn save(&self, other: Option<Self>) -> SaveAction<Self> {
        if let Some(other) = other {
            let server = ServerAction::<Self::DiffEndpoint>::new();
            server.dispatch(self.diff(other));

            return SaveAction::Diff(server);
        }

        let server = ServerAction::<Self::Endpoint>::new();
        server.dispatch(self.full());

        SaveAction::Full(server)
    }

    fn to_js_value(&self) -> wasm_bindgen::JsValue;
}

pub trait ToJsValue {
    const OBJECT_STORE: &'static str = "saving";
    type Error = AppError;

    fn to_js_value(&self) -> Result<wasm_bindgen::JsValue, Self::Error>;
}

pub trait OfflineSavableWithData: ToJsValue + Savable + Clone + Sized + 'static
where
    <<Self as OfflineSavableWithData>::Endpoint as ServerFn>::Output: Clone + std::marker::Sync,
{
    type Data = ();
    type Diff: ToJsValue = ();
    type Endpoint: ServerFn + Clone + Sync;

    fn full(&self, data: Self::Data) -> Self::Endpoint;
    fn diff(&self, other: &Self, data: Self::Data) -> Self::Diff;

    fn save(
        &self,
        other: Option<Self>,
        data: Self::Data,
    ) -> ServerAction<<Self as OfflineSavableWithData>::Endpoint> {
        // if let Some(other) = other {
        //     let server = ServerAction::<Self::DiffEndpoint>::new();
        //     server.dispatch(self.diff(other, data));
        //
        //     return SaveActionWithData::Diff(server);
        // }

        let server = ServerAction::<Self::Endpoint>::new();
        server.dispatch(self.full(data));

        return server;
    }
}

pub trait Loadable: Clone + Send + Sync {
    const OBJECT_STORE: &'static str = "saving";
    type SaveDiff: Loadable = ();
    type FetchData: ServerFn;

    fn fetch(&self) -> Self::FetchData;
    fn merge(&self, diff: Self::SaveDiff) -> Self;
    fn from_js_value(value: &wasm_bindgen::JsValue) -> Self;

    async fn load(data: Self::FetchData) -> Self {
        todo!()
    }

    fn load_blocking(data: Self::FetchData) -> Resource<Self> {
        todo!()
    }
}

impl Savable for () {}

impl ToJsValue for () {
    fn to_js_value(&self) -> AppResult<wasm_bindgen::JsValue> {
        Ok(wasm_bindgen::JsValue::NULL)
    }
}

impl OfflineSavable for () {
    type DiffEndpoint = VoidEndpoint;
    type Endpoint = VoidEndpoint;

    fn full(&self) -> Self::Endpoint {
        VoidEndpoint {}
    }

    fn diff(&self, _: Self) -> Self::DiffEndpoint {
        VoidEndpoint {}
    }

    fn to_js_value(&self) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::NULL
    }
}

impl OfflineSavableWithData for () {
    type Endpoint = VoidEndpoint;

    fn full(&self, data: ()) -> Self::Endpoint {
        VoidEndpoint {}
    }

    fn diff(&self, _: &Self, data: ()) -> () {
        ()
    }
}

impl Loadable for () {
    type SaveDiff = ();
    type FetchData = VoidEndpoint;

    fn merge(&self, diff: Self::SaveDiff) -> Self {
        todo!()
    }

    fn from_js_value(value: &wasm_bindgen::JsValue) -> Self {
        todo!()
    }

    fn fetch(&self) -> Self::FetchData {
        todo!()
    }
}

#[server(VoidEndpoint, "/api")]
pub async fn void_endpoint() -> Result<(), ServerFnError> {
    Ok(())
}

// TODO: Implement for Result<T, E>
//
// impl<T, E> Savable for Result<T, E>
// where
//     T: Send + Sync,
//     E: Send + Sync,
// {
// }
//
// impl<T, E> OfflineSavableWithData for Result<T, E>
// where
//     T: OfflineSavableWithData,
//     E: Clone + Send + Sync + 'static,
// {
//     type Data = T::Data;
//
//     type Diff = T::Diff;
//
//     type Endpoint = T::Endpoint;
//
//     fn full(&self, data: Self::Data) -> Self::Endpoint {
//         self?.full(data)
//     }
//
//     fn diff(&self, other: &Self, data: Self::Data) -> Self::Diff {
//         todo!()
//     }
//
//     fn to_js_value(&self) -> wasm_bindgen::JsValue {
//         todo!()
//     }
// }

impl<T, E> ToJsValue for Result<T, E>
where
    T: ToJsValue,
{
    const OBJECT_STORE: &'static str = T::OBJECT_STORE;

    fn to_js_value(&self) -> AppResult<wasm_bindgen::JsValue> {
        todo!()
    }
}

#[derive(Clone, Copy)]
pub struct SavingSignal<T, D>
where
    D: Clone + 'static,
    T: OfflineSavableWithData<Data = D>,
{
    signal: RwSignal<T>,
    data: D,
}

impl<T, D> leptos_reactive::SignalGet for SavingSignal<T, D>
where
    D: Clone + 'static,
    T: OfflineSavableWithData<Data = D>,
{
    type Value = T;

    fn get(&self) -> Self::Value {
        self.signal.get()
    }

    fn try_get(&self) -> Option<Self::Value> {
        self.signal.try_get()
    }
}

impl<T, D> leptos_reactive::SignalSet for SavingSignal<T, D>
where
    D: Clone + 'static,
    T: OfflineSavableWithData<Data = D>,
    <<T as saving::OfflineSavableWithData>::Endpoint as leptos::server_fn::ServerFn>::Output: Clone,
    <<T as saving::OfflineSavableWithData>::Endpoint as leptos::server_fn::ServerFn>::Error: Clone,
    <T as saving::ToJsValue>::Error:
        std::convert::From<indexed_db::Error<<T as saving::ToJsValue>::Error>>,
    <<T as saving::OfflineSavableWithData>::Diff as saving::ToJsValue>::Error: std::convert::From<
            indexed_db::Error<
                <<T as saving::OfflineSavableWithData>::Diff as saving::ToJsValue>::Error,
            >,
        >,
{
    type Value = T;

    fn set(&self, new_value: Self::Value) {
        let old_value = StoredValue::new_local(self.signal.get_untracked());
        let new_value = StoredValue::new_local(new_value);
        let data = StoredValue::new_local(self.data.clone());

        let action = ServerAction::<T::Endpoint>::new();

        Effect::new(move || {
            match action.value().get() {
                Some(Ok(_)) => {}
                Some(Err(_)) => {
                    let diff = new_value
                        .get_value()
                        .diff(&old_value.get_value(), data.get_value());

                    save_object_store(diff);
                }
                None => {}
            };
        });

        action.dispatch(new_value.get_value().full(data.get_value()));

        self.signal.set(new_value.get_value())
    }

    fn try_set(&self, new_value: Self::Value) -> Option<Self::Value> {
        self.signal.try_set(new_value)
    }
}

impl<T, D> SavingSignal<T, D>
where
    D: Clone + 'static,
    T: OfflineSavableWithData<Data = D>,
    <<T as saving::OfflineSavableWithData>::Endpoint as leptos::server_fn::ServerFn>::Output: Clone,
    <<T as saving::OfflineSavableWithData>::Endpoint as leptos::server_fn::ServerFn>::Error: Clone,
    <T as saving::ToJsValue>::Error:
        std::convert::From<indexed_db::Error<<T as saving::ToJsValue>::Error>>,
    <<T as saving::OfflineSavableWithData>::Diff as saving::ToJsValue>::Error: std::convert::From<
            indexed_db::Error<
                <<T as saving::OfflineSavableWithData>::Diff as saving::ToJsValue>::Error,
            >,
        >,
{
    pub fn new(signal: RwSignal<T>, data: D) -> Self {
        Self { signal, data }
    }
}

pub async fn save_object_store<T: ToJsValue + 'static>(value: T) -> Result<(), T::Error>
where
    <T as saving::ToJsValue>::Error:
        std::convert::From<indexed_db::Error<<T as saving::ToJsValue>::Error>>,
{
    let factory = indexed_db::Factory::<T::Error>::get()?;
    let db = factory.open_latest_version("TallyWeb").await?;

    db.transaction(&[T::OBJECT_STORE])
        .rw()
        .run(move |tr| {
            let obj = tr.object_store(T::OBJECT_STORE);

            async move {
                obj?.put(&value.to_js_value()?);
                Ok(())
            }
        })
        .await?;

    Ok(())
}
