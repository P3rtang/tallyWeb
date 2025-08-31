#![allow(unused_braces)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use leptos::{ev::MouseEvent, prelude::*};

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionModel<SelectionKey, T>
where
    SelectionKey: Clone + Debug + Send + Sync + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Send + Sync + 'static,
{
    items: HashMap<SelectionKey, TreeNode<T, SelectionKey>>,
    selection: HashMap<SelectionKey, bool>,
    multi_select: bool,
}

impl<SelectionKey, T> Default for SelectionModel<SelectionKey, T>
where
    SelectionKey: Clone + Debug + Send + Sync + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<SelectionKey, T> SelectionModel<SelectionKey, T>
where
    SelectionKey: Clone + Debug + Send + Sync + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            selection: HashMap::new(),
            multi_select: false,
        }
    }

    pub fn set_multi_select(&mut self, multi_select: bool) {
        self.multi_select = multi_select
    }

    pub fn get(&self, key: &SelectionKey) -> Option<&T> {
        Some(&self.items.get(key)?.row)
    }

    pub fn get_mut(&mut self, key: &SelectionKey) -> Option<&mut T> {
        Some(&mut self.items.get_mut(key)?.row)
    }

    pub fn get_node(&self, key: &SelectionKey) -> Option<&TreeNode<T, SelectionKey>> {
        self.items.get(key)
    }

    pub fn get_node_mut(&mut self, key: &SelectionKey) -> Option<&mut TreeNode<T, SelectionKey>> {
        self.items.get_mut(key)
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn select(&mut self, key: &SelectionKey) {
        if !self.multi_select {
            self.selection.clear();
        }
        self.selection.insert(key.clone(), true);
    }

    pub fn toggle(&mut self, key: &SelectionKey) {
        let current_value = self.is_selected(key);
        if !self.multi_select {
            self.selection.clear();
        }

        self.selection.insert(key.clone(), !current_value);
    }

    pub fn selection_mut(&mut self) -> Vec<&mut T> {
        let selected = self.selection.clone();
        self.items
            .iter_mut()
            .filter(|(key, _)| selected.get(*key).cloned().unwrap_or_default())
            .map(|(_, item)| &mut item.row)
            .collect()
    }

    pub fn selection(&self) -> Vec<&T> {
        self.selection
            .iter()
            .filter(|(_, b)| **b)
            .filter_map(|(k, _)| self.items.get(k).map(|i| &i.row))
            .collect()
    }

    pub fn get_selected_keys(&self) -> Vec<&SelectionKey> {
        self.selection
            .iter()
            .filter(|(k, b)| **b && self.items.contains_key(k))
            .map(|(key, _)| key)
            .collect()
    }

    pub fn get_owned_selected_keys(&self) -> Vec<SelectionKey> {
        self.selection
            .iter()
            .filter(|(k, b)| **b && self.items.contains_key(k))
            .map(|(key, _)| key)
            .cloned()
            .collect()
    }

    pub fn remove_item(&mut self, key: &SelectionKey) -> Option<T> {
        Some(self.items.remove(key)?.row)
    }

    pub fn is_selected(&self, key: &SelectionKey) -> bool {
        self.selection.get(key).cloned().unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.selection.is_empty()
    }
}

#[component]
pub fn TreeViewWidget<T, F, SelectionKey, FV, IV, EC>(
    each: F,
    key: fn(&T) -> SelectionKey,
    each_child: EC,
    view: FV,
    #[prop(default=create_signal(false).0.into(), into)] show_separator: Signal<bool>,
    #[prop(default=create_rw_signal(SelectionModel::default()), into)] selection_model: RwSignal<
        SelectionModel<SelectionKey, T>,
    >,
    #[prop(optional)] on_click: Option<fn(&SelectionKey, MouseEvent)>,
) -> impl IntoView
where
    SelectionKey: Clone + Debug + Send + Sync + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Send + Sync + 'static,
    IV: IntoView + 'static,
    F: Fn() -> Vec<T> + Copy + Sync + Send + 'static,
    FV: Fn(&T) -> IV + Copy + Sync + Send + 'static,
    EC: Fn(&T) -> Vec<T> + Copy + Sync + Send + 'static,
{
    let nodes = Memo::new(move |_| each());

    Effect::new_isomorphic(move |_| {
        each().into_iter().for_each(move |c| {
            let key_val = StoredValue::new(key(&c));
            if selection_model
                .get_untracked()
                .get_node(&key_val.get_value())
                .is_none()
            {
                let node = TreeNode::<T, SelectionKey>::new(key, c, 0);
                selection_model.update(move |s| {
                    s.items.insert(key_val.get_value(), node);
                });
            }
        })
    });

    let each = move || {
        nodes()
            .iter()
            .filter_map(|n| selection_model.get_untracked().get_node(&key(n)).cloned())
            .collect::<Vec<_>>()
    };

    view! {
        <tree-view>
            <ul>
                <For
                    each
                    key=move |c| key(&c.row)
                    children=move |item| {
                        view! {
                            <TreeViewRow
                                item=item.row.clone()
                                key
                                selection_model
                                view
                                each_child
                                on_click
                            >
                                {view(&item.row)}
                            </TreeViewRow>
                            <Show when=show_separator fallback=|| ()>
                                <hr />
                            </Show>
                        }
                    }
                />

            </ul>
        </tree-view>
    }
    .into_view()
}

#[component]
fn TreeViewRow<T, SelectionKey, FV, IV, EC>(
    children: ChildrenFn,
    item: T,
    key: fn(&T) -> SelectionKey,
    each_child: EC,
    view: FV,
    selection_model: RwSignal<SelectionModel<SelectionKey, T>>,
    on_click: Option<fn(&SelectionKey, MouseEvent)>,
) -> impl IntoView
where
    SelectionKey: Clone + Debug + Send + Sync + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Send + Sync + 'static,
    FV: Fn(&T) -> IV + Copy + Send + Sync + 'static,
    IV: IntoView + 'static,
    EC: Fn(&T) -> Vec<T> + Copy + Send + Sync + 'static,
{
    let key_val = StoredValue::new(key(&item));

    let node = create_read_slice(selection_model, move |sm| {
        sm.items.get(&key_val.get_value()).cloned()
    });

    let (is_expanded, toggle_expand) = create_slice(
        selection_model,
        move |model| {
            model
                .items
                .get(&key_val.get_value())
                .map(|n| n.is_expanded)
                .unwrap_or_default()
        },
        move |model, _| {
            if let Some(node) = model.items.get_mut(&key_val.get_value()) {
                node.toggle_expand()
            };
        },
    );

    let (is_selected, set_selected) = create_slice(
        selection_model,
        move |model| model.is_selected(&key_val.get_value()),
        move |model, _| model.select(&key_val.get_value()),
    );

    let caret_class = move || "caret fa-solid fa-caret-right";

    let background = Memo::new(move |_| {
        if is_selected() {
            "var(--accent, #3584E4)"
        } else {
            "none"
        }
    });

    let on_row_click = move |_: MouseEvent| set_selected(());

    let on_caret_click = move |ev: MouseEvent| {
        ev.stop_propagation();
        toggle_expand(())
    };

    let depth = move || node().map(|n| n.depth).unwrap_or_default();

    let node_children = Memo::new(move |_| each_child(&item));

    Effect::new_isomorphic(move |_| {
        node_children().into_iter().for_each(|c| {
            let key_val = StoredValue::new(key(&c));
            if selection_model
                .get_untracked()
                .get_node(&key_val.get_value())
                .is_none()
            {
                let node = TreeNode::<T, SelectionKey>::new(key, c, depth() + 1);
                selection_model.update(|s| {
                    s.items.insert(key_val.get_value(), node);
                });
            }
        });
    });

    let children = StoredValue::new(children);

    view! {
        <li style:display="block">
            <div
                style:padding-left=move || format!("{}em", 2.0 * depth() as f64)
                style:background=move || background()
                style:display="flex"
                class:selectable=true
                class:row=true
                class:selected=is_selected
                on:click=move |ev| {
                    if let Some(f) = on_click {
                        if let Some(k) = key_val.try_get_value() {
                            f(&k, ev);
                        }
                    } else {
                        on_row_click(ev);
                    }
                }
            >

                <Show when=move || {
                    node.try_get_untracked()
                        .flatten()
                        .is_some_and(|c| !each_child(&c.row).is_empty())
                }>
                    <div
                        class=caret_class
                        style:transform=move || if is_expanded() { "rotate(90deg)" } else { "" }
                        style:cursor="pointer"
                        style:font-size="24px"
                        style:transition="transform 0.24s"
                        on:click=on_caret_click
                    ></div>
                </Show>
                {children.get_value()()}
            </div>
            <ul style:display=move || if is_expanded() { "block" } else { "none" }>
                <For
                    each=node_children
                    key
                    children=move |item| {
                        view! {
                            <TreeViewRow
                                key
                                item=item.clone()
                                selection_model=selection_model
                                each_child=each_child
                                view=view
                                on_click
                            >
                                {view(&item)}
                            </TreeViewRow>
                        }
                    }
                />

            </ul>
        </li>
    }
    .into_any()
}

#[derive(Debug, Clone)]
pub struct TreeNode<T, SelectionKey>
where
    SelectionKey: Clone + Debug + Send + Sync + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Send + Sync + 'static,
{
    pub key: fn(&T) -> SelectionKey,
    pub row: T,
    pub depth: usize,
    pub is_expanded: bool,
}

impl<T, SelectionKey> PartialEq for TreeNode<T, SelectionKey>
where
    SelectionKey: Clone + Debug + Send + Sync + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        (self.key)(&self.row) == (other.key)(&other.row)
            && self.depth == other.depth
            && self.is_expanded == other.is_expanded
    }
}

impl<T, SelectionKey> TreeNode<T, SelectionKey>
where
    SelectionKey: Clone + Debug + Send + Sync + PartialEq + Eq + Hash + 'static,
    T: Debug + Clone + PartialEq + Send + Sync + 'static,
{
    pub fn new(key: fn(&T) -> SelectionKey, item: T, depth: usize) -> Self {
        Self {
            key,
            row: item.clone(),
            depth,
            is_expanded: false,
        }
    }

    pub fn set_expand(&mut self, do_expand: bool) {
        self.is_expanded = do_expand
    }

    pub fn toggle_expand(&mut self) {
        self.is_expanded = !self.is_expanded
    }
}
