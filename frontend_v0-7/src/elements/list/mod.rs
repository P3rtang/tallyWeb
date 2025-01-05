use components::{Caret, CaretState, ChildWrapper, RowWrapper, Separator, Tree, WrappedRowState};
use leptos::{
    attr::{any_attribute::IntoAnyAttribute, Href},
    either::Either,
    ev,
    prelude::{Await, *},
};
use std::{collections::HashSet, sync::Arc};

mod row;

pub use row::RowSlot;
pub(crate) use row::{ListChildren, ListItemChildren};

stylance::import_style!(style, "./list.module.scss");

#[component]
pub fn List<T, I, EF, K, KF>(
    each: EF,
    key: KF,
    #[prop(optional)] children: Option<ListChildren<T, I>>,

    // slots
    row_slot: RowSlot<T, K>,
    #[prop(optional)] separator: Option<Separator>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
    EF: Fn() -> I + Send + Sync + Clone + 'static,
    I: IntoIterator<Item = T> + Send + 'static,
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
{
    let each = StoredValue::new(each);
    let key = StoredValue::new(key);
    let row_slot = StoredValue::new(row_slot);

    let list_children = move |row: T| (row_slot.get_value().children.0)(row);

    let (expanded, set_expanded) = signal(HashSet::<K>::new());

    let is_expanded = move |key: K| expanded.get().contains(&key);

    let handle_expand = move |key: K| {
        set_expanded.update(|set| {
            if !set.remove(&key) {
                set.insert(key);
            }
        })
    };

    let caret_transform = move |is_expanded: bool| {
        if is_expanded {
            "transform: rotate(90deg)"
        } else {
            ""
        }
    };

    let row_wrapper = move |wrapped: WrappedRowState<_>| {
        let key: K = wrapped.key;
        let is_selected = move || (row_slot.get_value().is_selected.0)(key.clone());

        view! {
            <li class:selected=is_selected class=style::row>{wrapped.view}</li>
        }
    };

    let caret_children = move |state: CaretState<_>| {
        let key = StoredValue::new(state.key);

        view! {
            <div class=style::caret>
                <img
                    style=move || caret_transform(state.is_expanded)
                    height="20px"
                    width="20px"
                    class:hidden=move || (row_slot.get_value().is_selected.0)(key.get_value())
                    src="/icons/caret-right-fill-svgrepo-com-white.svg"
                />
                <img
                    style=move || caret_transform(state.is_expanded)
                    height="20px"
                    width="20px"
                    class:hidden=move || !(row_slot.get_value().is_selected.0)(key.get_value())
                    src="/icons/caret-right-fill-svgrepo-com.svg"
                />
            </div>
        }
    };

    let attrs = move |state: CaretState<_>| {
        let class = if (row_slot.get_value().is_selected.0)(state.key) {
            "hover-darken"
        } else {
            "hover-lighten"
        };

        view! {<{..} attr:class=class />}.into_any_attr()
    };

    if let Some(children) = children {
        Either::Left(view! {
            <ul class=style::list>
                <Tree
                    each=each.get_value()
                    key=key.get_value()
                    children=move |row| (children.0)(row.clone())
                    view=move |row| list_children(row.clone())
                    separator
                >
                    <ChildWrapper let:wrapped slot>
                        <ul>{wrapped}</ul>
                    </ChildWrapper>

                    <RowWrapper indent_size=48 children=row_wrapper slot/>

                    <Caret attrs is_expanded on_expand=handle_expand children=caret_children slot />
                </Tree>
            </ul>
        })
    } else {
        Either::Right(view! {
            <ul class=style::list>
                <For
                    each=each.get_value()
                    key=key.get_value()
                    children=move |row| list_children(row.clone())
                />
            </ul>
        })
    }
}
