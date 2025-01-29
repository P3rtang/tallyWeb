use std::sync::Arc;

use leptos::{attr::Href, ev, prelude::*};

#[derive(Clone)]
#[slot]
pub struct RowSlot<T, K>
where
    K: Send + Sync + 'static,
    T: Clone + Send + Sync,
{
    #[prop(into, optional)]
    href: Option<Href>,

    #[prop(into, default=Arc::new(move |_| ()))]
    on_click: Arc<dyn Fn(ev::MouseEvent) + Send + Sync>,

    #[prop(into, optional)]
    is_selected: IsSelected<K>,

    #[prop(into)]
    children: ListItemChildren<T>,
}

#[derive(Clone)]
pub struct ListItemChildren<T>(pub(crate) Arc<dyn Fn(T) -> AnyView + Send + Sync + 'static>)
where
    T: Clone + Send + Sync;

impl<F, T> From<F> for ListItemChildren<T>
where
    F: Fn(T) -> AnyView + Send + Sync + 'static,
    T: Clone + Send + Sync,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

#[derive(Clone)]
pub struct ListChildren<T, I>(pub(crate) Arc<dyn Fn(T) -> I + Send + Sync + 'static>)
where
    T: Clone + Send + Sync,
    I: IntoIterator<Item = T> + Send + Clone + 'static;

impl<F, T, I> From<F> for ListChildren<T, I>
where
    F: Fn(T) -> I + Send + Sync + 'static,
    T: Clone + Send + Sync,
    I: IntoIterator<Item = T> + Send + Clone + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

#[derive(Clone)]
pub struct IsSelected<K>(pub(crate) Arc<dyn Fn(K) -> bool + Send + Sync + 'static>)
where
    K: Send + Sync + 'static;

impl<F, K> From<F> for IsSelected<K>
where
    K: Send + Sync + 'static,
    F: Fn(K) -> bool + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

impl<K> Default for IsSelected<K>
where
    K: Send + Sync + 'static,
{
    fn default() -> Self {
        Self(Arc::new(|_| false))
    }
}
