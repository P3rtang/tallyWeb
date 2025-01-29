use super::Caret;
use leptos::prelude::*;
use std::{hash::Hash, sync::Arc};

#[component]
pub fn Tree<IF, I, EF, N, KF, K, T, CF>(
    each: IF,
    key: KF,
    children: CF,
    view: EF,

    #[prop(optional)] caret: Caret<K>,
    #[prop(optional)] child_wrapper: ChildWrapper,
    #[prop(optional)] row_wrapper: RowWrapper<K>,
    #[prop(optional_no_strip)] separator: Option<Separator>,
) -> impl IntoView
where
    IF: Fn() -> I + Send + Sync + 'static,
    I: IntoIterator<Item = T> + Send + 'static,
    EF: Fn(&T) -> N + Send + Sync + Clone + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
    K: Clone + Eq + Hash + Send + 'static,
    T: Clone + Send + Sync + 'static,
    CF: Fn(&T) -> I + Send + Sync + Clone + 'static,
{
    let separator = StoredValue::new(separator);
    let row_wrapper = StoredValue::new(row_wrapper);

    view! {
        <For
            each
            key=key.clone()
            children=move |row| {
                view! {
                    <TreeRow
                        caret=caret.clone()
                        row
                        key=key.clone()
                        view=view.clone()
                        children=children.clone()
                        child_wrapper=child_wrapper.clone()
                        row_wrapper=row_wrapper.get_value()
                    />
                    <Show when=move || {
                        separator.get_value().is_some()
                    }>
                        {(row_wrapper
                            .get_value()
                            .children
                            .0)((((separator.get_value().unwrap().children)()), 0).into())}
                    </Show>
                }
            }
        />
    }
}

#[derive(Clone)]
#[slot]
pub struct ChildWrapper {
    #[prop(into)]
    children: Wrapped,
}

impl Default for ChildWrapper {
    fn default() -> Self {
        Self {
            children: (move |v| v).into(),
        }
    }
}

#[derive(Clone)]
#[slot]
pub struct RowWrapper<K>
where
    K: Eq + Hash + Send + 'static,
{
    #[prop(optional, into)]
    indent_size: Option<Signal<usize>>,

    #[prop(into)]
    children: WrappedRow<K>,
}

impl<K> Default for RowWrapper<K>
where
    K: Eq + Hash + Send + 'static,
{
    fn default() -> Self {
        Self {
            indent_size: None,
            children: (move |v: WrappedRowState<K>| v.view).into(),
        }
    }
}

#[derive(Clone)]
pub struct WrappedRow<K>(Arc<dyn Fn(WrappedRowState<K>) -> AnyView + Send + Sync + 'static>)
where
    K: Eq + Hash + Send + 'static;

impl<F, IV, K> From<F> for WrappedRow<K>
where
    F: Fn(WrappedRowState<K>) -> IV + Send + Sync + 'static,
    IV: IntoView + Send + 'static,
    K: Eq + Hash + Send + 'static,
{
    fn from(value: F) -> Self {
        Self(std::sync::Arc::new(move |wrapped| {
            value(wrapped).into_any()
        }))
    }
}

pub struct WrappedRowState<K>
where
    K: Eq + Hash + 'static,
{
    pub view: AnyView,
    pub depth: usize,
    pub key: Option<K>,
}

impl<K> From<(AnyView, usize, K)> for WrappedRowState<K>
where
    K: Eq + Hash + 'static,
{
    fn from(value: (AnyView, usize, K)) -> Self {
        Self {
            view: value.0,
            depth: value.1,
            key: Some(value.2),
        }
    }
}

impl<K> From<(AnyView, usize)> for WrappedRowState<K>
where
    K: Eq + Hash + 'static,
{
    fn from(value: (AnyView, usize)) -> Self {
        Self {
            view: value.0,
            depth: value.1,
            key: None,
        }
    }
}

#[derive(Clone)]
pub struct Wrapped(Arc<dyn Fn(AnyView) -> AnyView + Send + Sync + 'static>);

impl<T, IV> From<T> for Wrapped
where
    T: Fn(AnyView) -> IV + Send + Sync + 'static,
    IV: IntoView + Send + 'static,
{
    fn from(value: T) -> Self {
        Self(std::sync::Arc::new(move |wrapped| {
            value(wrapped).into_any()
        }))
    }
}

#[derive(Clone)]
#[slot]
pub struct Separator {
    children: ChildrenFn,
}

#[component]
pub fn TreeRow<I, EF, N, KF, K, T, CF>(
    row: T,
    key: KF,
    view: EF,
    children: CF,

    #[prop(default = 0)] depth: usize,

    #[prop(optional_no_strip)] caret: Caret<K>,
    #[prop(optional)] child_wrapper: ChildWrapper,
    #[prop(optional)] row_wrapper: RowWrapper<K>,
) -> impl IntoView
where
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
    K: Clone + Eq + Hash + Send + 'static,
    T: Clone + Send + Sync + 'static,
    EF: Fn(&T) -> N + Send + Sync + Clone + 'static,
    N: IntoView + 'static,
    CF: Fn(&T) -> I + Send + Sync + Clone + 'static,
    I: IntoIterator<Item = T> + Send + 'static,
{
    let child_wrapper = StoredValue::new(child_wrapper);
    let row_wrapper = StoredValue::new(row_wrapper);
    let children = StoredValue::new(children);
    let caret = StoredValue::new(caret);
    let row = StoredValue::new(row);
    let key_fn = StoredValue::new(key);
    let key = move || key_fn.get_value()(&row.get_value());

    let is_expanded_uncontrolled = RwSignal::new(false);

    let is_expanded = move || (caret.get_value().is_expanded.0)(key());

    let caret_state = move || (key(), is_expanded()).into();
    let child = move || (caret.get_value().children.0)(caret_state());

    let handle_click = {
        move |_| {
            if let Some(func) = caret.get_value().on_expand.map(|oe| oe.0) {
                func(key())
            }

            is_expanded_uncontrolled.set(!is_expanded_uncontrolled.get())
        }
    };
    let child = StoredValue::new(child);
    let handle_click = StoredValue::new(handle_click);

    let child_views = {
        let key = key_fn;
        let view = view.clone();
        std::sync::Arc::new(move |row| {
            view! {
                <TreeRow
                    caret=caret.get_value()
                    row
                    key=key.get_value()
                    view=view.clone()
                    children=children.get_value()
                    depth=depth + 1
                    child_wrapper=child_wrapper.get_value()
                    row_wrapper=row_wrapper.get_value()
                />
            }
        })
    };

    let child_views = StoredValue::new(child_views);

    let has_children = move || {
        children.get_value()(&row.get_value())
            .into_iter()
            .next()
            .is_some()
    };

    let show_children = move || {
        if has_children() && is_expanded() {
            "block"
        } else {
            "none"
        }
    };

    // TODO: add differing indent on whether there is a caret
    let padding = move || {
        if let Some(indent) = row_wrapper.get_value().indent_size {
            format!("{}px", indent.get() * depth)
        } else {
            "".to_string()
        }
    };

    (row_wrapper.get_value().children.0)(
        (
            view! {
                <div style:padding-left=padding>
                    <Show when=has_children>
                        <button
                            {..(caret.get_value().attrs.0)(caret_state())}
                            on:click=handle_click.get_value()
                        >
                            {child.get_value()()}
                        </button>
                    </Show>
                    {view(&row.get_value())}
                </div>
                <div style:display=show_children>
                    {(child_wrapper
                        .get_value()
                        .children
                        .0)(
                        view! {
                            <For
                                each=move || children.get_value()(&row.get_value())
                                key=key_fn.get_value()
                                children=move |row| child_views.get_value()(row)
                            />
                        }
                            .into_any(),
                    )}
                </div>
            }
            .into_any(),
            depth,
            key(),
        )
            .into(),
    )
}
