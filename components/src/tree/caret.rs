use std::{hash::Hash, sync::Arc};

use leptos::{
    attr::{
        any_attribute::{AnyAttribute, IntoAnyAttribute},
        Attribute,
    },
    prelude::*,
};

#[derive(Clone)]
#[slot]
pub struct Caret<K>
where
    K: Eq + Hash + Send + 'static,
{
    #[prop(default=false.into(), into)]
    is_expanded: IsExpanded<K>,

    #[prop(into, optional)]
    on_expand: Option<OnExpand<K>>,

    #[prop(into, default=(move |_| view!{<{..} />}).into())]
    attrs: CaretAttrs<K>,

    #[prop(into)]
    children: CaretChild<K>,
}

impl<K> Default for Caret<K>
where
    K: Eq + Hash + Send + 'static,
{
    fn default() -> Self {
        let caret_transform = move |is_expanded: bool| {
            if !is_expanded {
                "rotate(-90deg)"
            } else {
                ""
            }
        };

        Self {
            is_expanded: false.into(),
            on_expand: None,
            attrs: (move |_| ()).into(),
            children: (move |state: CaretState<K>| {
                view! { <div style:transform=caret_transform(state.is_expanded)>{"▼"}</div> }
            })
            .into(),
        }
    }
}

#[derive(Clone)]
pub struct IsExpanded<K>(pub(crate) Arc<dyn Fn(K) -> bool + Send + Sync + 'static>)
where
    K: Eq + Hash + 'static;

impl<F, K> From<F> for IsExpanded<K>
where
    F: Fn(K) -> bool + Send + Sync + 'static,
    K: Eq + Hash + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

impl<K> From<bool> for IsExpanded<K>
where
    K: Eq + Hash + 'static,
{
    fn from(value: bool) -> Self {
        Self(Arc::new(move |_| value))
    }
}

impl<K> From<Signal<bool>> for IsExpanded<K>
where
    K: Eq + Hash + 'static,
{
    fn from(value: Signal<bool>) -> Self {
        Self(Arc::new(move |_| value.get()))
    }
}

#[derive(Clone)]
pub struct OnExpand<K>(pub(crate) Arc<dyn Fn(K) + Send + Sync + 'static>)
where
    K: Eq + Hash + 'static;

impl<F, K> From<F> for OnExpand<K>
where
    F: Fn(K) + Send + Sync + 'static,
    K: Eq + Hash + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

#[derive(Clone)]
pub struct CaretChild<K>(pub(crate) Arc<dyn Fn(CaretState<K>) -> AnyView + Send + Sync + 'static>)
where
    K: Eq + Hash + Send + 'static;

impl<F, IV, K> From<F> for CaretChild<K>
where
    F: Fn(CaretState<K>) -> IV + Send + Sync + 'static,
    IV: IntoView + 'static,
    K: Eq + Hash + Send + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(move |v| value(v).into_any()))
    }
}

#[derive(Clone)]
pub struct CaretState<K>
where
    K: Eq + Hash + Send + 'static,
{
    pub key: K,
    pub is_expanded: bool,
}

impl<K> From<(K, bool)> for CaretState<K>
where
    K: Eq + Hash + Send + 'static,
{
    fn from(value: (K, bool)) -> Self {
        Self {
            key: value.0,
            is_expanded: value.1,
        }
    }
}

#[derive(Clone)]
pub struct CaretAttrs<K>(
    pub(crate) Arc<dyn Fn(CaretState<K>) -> AnyAttribute + Sync + Send + 'static>,
)
where
    K: Eq + Hash + Send + 'static;

impl<K, F, A> From<F> for CaretAttrs<K>
where
    K: Eq + Hash + Send + 'static,
    F: Fn(CaretState<K>) -> A + Send + Sync + 'static,
    A: Attribute + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(move |state| value(state).into_any_attr()))
    }
}
