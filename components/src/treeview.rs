#![allow(unused_braces)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use leptos::{ev::MouseEvent, *};

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionModel<S, T>
where
    S: Clone + PartialEq + Eq + Hash + 'static,
    T: Clone + 'static + Debug + PartialEq,
{
    items: HashMap<S, TreeNode<T, S>>,
    selection: HashMap<S, bool>,
    multi_select: bool,
}

impl<S, T> Default for SelectionModel<S, T>
where
    S: Clone + PartialEq + Eq + Hash + 'static,
    T: Clone + 'static + Debug + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S, T> SelectionModel<S, T>
where
    S: Clone + PartialEq + Eq + Hash + 'static,
    T: Clone + 'static + Debug + PartialEq,
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

    pub fn get(&self, key: &S) -> Option<&T> {
        Some(&self.items.get(key)?.row)
    }

    pub fn get_mut(&mut self, key: &S) -> Option<&mut T> {
        Some(&mut self.items.get_mut(key)?.row)
    }

    pub fn get_node(&self, key: &S) -> Option<&TreeNode<T, S>> {
        self.items.get(key)
    }

    pub fn get_node_mut(&mut self, key: &S) -> Option<&mut TreeNode<T, S>> {
        self.items.get_mut(key)
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn select(&mut self, key: &S) {
        if !self.multi_select {
            self.selection.clear();
        }
        self.selection.insert(key.clone(), true);
    }

    pub fn toggle(&mut self, key: &S) {
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

    pub fn get_selected_keys(&self) -> Vec<&S> {
        self.selection
            .iter()
            .filter(|(k, b)| **b && self.items.contains_key(k))
            .map(|(key, _)| key)
            .collect()
    }

    pub fn get_owned_selected_keys(&self) -> Vec<S> {
        self.selection
            .iter()
            .filter(|(k, b)| **b && self.items.contains_key(k))
            .map(|(key, _)| key)
            .cloned()
            .collect()
    }

    pub fn remove_item(&mut self, key: &S) -> Option<T> {
        Some(self.items.remove(key)?.row)
    }

    pub fn is_selected(&self, key: &S) -> bool {
        return self.selection.get(key).cloned().unwrap_or_default();
    }

    pub fn is_empty(&self) -> bool {
        self.selection.is_empty()
    }
}

#[component]
pub fn TreeViewWidget<T, F, S, FV, IV, EC>(
    each: F,
    key: fn(&T) -> S,
    each_child: EC,
    view: FV,
    #[prop(default=create_signal(false).0.into(), into)] show_separator: Signal<bool>,
    #[prop(default=create_rw_signal(SelectionModel::default()), into)] selection_model: RwSignal<
        SelectionModel<S, T>,
    >,
    #[prop(optional)] on_click: Option<fn(&S, MouseEvent)>,
) -> impl IntoView
where
    T: Debug + Clone + PartialEq + 'static,
    S: Debug + Clone + PartialEq + Eq + Hash + ToString + 'static,
    F: Fn() -> Vec<T> + Copy + 'static,
    FV: Fn(&T) -> IV + Copy + 'static,
    IV: IntoView,
    EC: Fn(&T) -> Vec<T> + Copy + 'static,
{
    let nodes = create_memo(move |_| each());

    create_isomorphic_effect(move |_| {
        each().into_iter().for_each(move |c| {
            let key_val = store_value(key(&c));
            if selection_model
                .get_untracked()
                .get_node(&key_val())
                .is_none()
            {
                let node = TreeNode::<T, S>::new(key, c, 0);
                selection_model.update(move |s| {
                    s.items.insert(key_val(), node);
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
                                <hr/>
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
fn TreeViewRow<T, S, FV, IV, EC>(
    children: ChildrenFn,
    item: T,
    key: fn(&T) -> S,
    each_child: EC,
    view: FV,
    selection_model: RwSignal<SelectionModel<S, T>>,
    on_click: Option<fn(&S, MouseEvent)>,
) -> impl IntoView
where
    T: Debug + Clone + PartialEq + 'static,
    S: Debug + Clone + PartialEq + Eq + Hash + ToString + 'static,
    FV: Fn(&T) -> IV + Copy + 'static,
    IV: IntoView,
    EC: Fn(&T) -> Vec<T> + Copy + 'static,
{
    let key_val = store_value(key(&item));

    let node = create_read_slice(selection_model, move |sm| sm.items.get(&key_val()).cloned());

    let (is_expanded, toggle_expand) = create_slice(
        selection_model,
        move |model| {
            model
                .items
                .get(&key_val())
                .map(|n| n.is_expanded)
                .unwrap_or_default()
        },
        move |model, _| {
            if let Some(node) = model.items.get_mut(&key_val()) {
                node.toggle_expand()
            };
        },
    );

    let (is_selected, set_selected) = create_slice(
        selection_model,
        move |model| model.is_selected(&key_val()),
        move |model, _| model.select(&key_val()),
    );

    let caret_class = move || "caret fa-solid fa-caret-right";

    let div_class = move || {
        let mut class = String::from("selectable row");
        if is_selected() {
            class += " selected"
        }

        class
    };

    let background = create_memo(move |_| {
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

    let depth_style = move || {
        let margin = format!("{}em", 2.0 * depth() as f64);
        let style = format!("padding-left:{};", margin);
        style
    };

    let node_children = create_memo(move |_| each_child(&item));

    create_isomorphic_effect(move |_| {
        node_children().into_iter().for_each(|c| {
            let key_val = store_value(key(&c));
            if selection_model
                .get_untracked()
                .get_node(&key_val())
                .is_none()
            {
                let node = TreeNode::<T, S>::new(key, c, depth() + 1);
                selection_model.update(|s| {
                    s.items.insert(key_val(), node);
                });
            }
        });
    });

    let children = store_value(children);

    view! {
        <li style:display="block">
            <div
                style=depth_style
                style:background=background
                style:display="flex"
                class=div_class
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
                        style:transform=if is_expanded() { "rotate(90deg)" } else { "" }
                        style:cursor="pointer"
                        style:font-size="24px"
                        on:click=on_caret_click
                    ></div>
                </Show>
                {children()}
            </div>
            <ul style:display=move || if is_expanded() { "block" } else { "none" }>
                <For
                    each=node_children
                    key=move |item| key(&item)
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode<T, S>
where
    T: Clone + 'static + Debug + PartialEq,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub key: fn(&T) -> S,
    pub row: T,
    pub depth: usize,
    pub is_expanded: bool,
}

impl<T, S> TreeNode<T, S>
where
    T: Clone + 'static + Debug + PartialEq,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub fn new(key: fn(&T) -> S, item: T, depth: usize) -> Self {
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
