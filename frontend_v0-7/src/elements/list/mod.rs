use crate::elements::{Icon, IconColor, IconKind};
pub use components::Separator;
use components::{Caret, CaretState, ChildWrapper, RowWrapper, Tree, WrappedRowState};
use leptos::{attr::any_attribute::IntoAnyAttribute, either::Either, prelude::*};
use std::collections::HashSet;

mod row;

pub(crate) use row::ListChildren;
pub use row::RowSlot;

stylance::import_style!(style, "./list.module.scss");

#[component]
pub fn List<T, I, EF, K, KF>(
    each: EF,
    key: KF,
    #[prop(into, optional)] children: Option<ListChildren<T, I>>,

    // slots
    row_slot: RowSlot<T, K>,
    #[prop(optional)] separator: Option<Separator>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
    EF: Fn() -> I + Send + Sync + Clone + 'static,
    I: IntoIterator<Item = T> + Send + Clone + 'static,
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
{
    let each = StoredValue::new(each);
    let key = StoredValue::new(key);
    let row_slot = StoredValue::new(row_slot);
    let separator = StoredValue::new(separator);

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
        let key: Option<K> = wrapped.key;
        let is_selected = move || {
            key.clone()
                .is_some_and(|k| (row_slot.get_value().is_selected.0)(k))
        };

        view! {
            <li class:selected=is_selected class=style::row>{wrapped.view}</li>
        }
    };

    let caret_children = move |state: CaretState<_>| {
        let key = StoredValue::new(state.key);

        let color = Signal::derive(move || {
            if (row_slot.get_value().is_selected.0)(key.get_value()) {
                IconColor::Black
            } else {
                IconColor::White
            }
        });

        view! {
            <div class=style::caret style=move || caret_transform(state.is_expanded)>
                <Icon kind=IconKind::CaretRight color />
            </div>
        }
    };

    let attrs = move |state: CaretState<_>| {
        let class = if (row_slot.get_value().is_selected.0)(state.key) {
            "hover-darken"
        } else {
            "hover-lighten"
        };

        view! {<{..} attr:class=class attr:aria_label="expand tree" />}.into_any_attr()
    };

    if let Some(children) = children.clone() {
        Either::Left(view! {
            <ul class=style::list>
                <Tree
                    each=each.get_value()
                    key=key.get_value()
                    children=move |row| (children.0)(row.clone())
                    view=move |row| list_children(row.clone())
                    separator=separator.get_value()
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
                    children=move |row| {
                        let row = StoredValue::new(row);
                        let is_selected = move || (row_slot.get_value().is_selected.0)(key.get_value()(&row.get_value()));

                        view!{
                            <li
                                class:selected=is_selected
                                class=style::row
                            >
                                <div style:padding-left="12px">{list_children(row.get_value())}</div>
                            </li>
                        }
                    }
                />
            </ul>
        })
    }
}
