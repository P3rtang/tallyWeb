#![allow(unused_braces)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use leptos::{ev::MouseEvent, *};

#[derive(Debug, Clone)]
pub struct SelectionModel<T, S>
where
    T: Clone + 'static + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    items: HashMap<S, TreeNode<T, S>>,
    selection: HashMap<S, bool>,
    pub selected: RwSignal<Vec<RwSignal<T>>>,
}

impl<T, S> SelectionModel<T, S>
where
    T: Clone + 'static + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub fn new(selected: RwSignal<Vec<RwSignal<T>>>) -> Self {
        return Self {
            items: HashMap::new(),
            selection: HashMap::new(),
            selected,
        };
    }

    pub fn select(&mut self, key: &S) {
        let current_value = self.selection.get(key).cloned().unwrap_or_default();
        self.selection.clear();
        self.selection.insert(key.clone(), !current_value);

        self.selected.update(|s| {
            *s = self
                .selection()
                .into_iter()
                .map(|t| create_rw_signal(t))
                .collect()
        })
    }

    pub fn selection(&self) -> Vec<T> {
        self.selection
            .clone()
            .into_iter()
            .filter(|(_, b)| *b)
            .map(|(k, _)| self.items.get(&k).cloned().unwrap().row)
            .collect()
    }

    pub fn get_selected(&self) -> RwSignal<Vec<RwSignal<T>>> {
        self.selected
    }

    pub fn get_selected_keys(&self) -> Vec<&S> {
        self.selection
            .iter()
            .filter(|(_, b)| **b)
            .map(|(key, _)| key)
            .collect()
    }

    pub fn is_selected(&self, key: &S) -> bool {
        return self.selection.get(key).cloned().unwrap_or_default();
    }

    pub fn is_empty(&self) -> bool {
        self.selection().is_empty()
    }
}

#[component]
pub fn TreeViewWidget<T, F, S, FS>(
    each: F,
    key: fn(&T) -> S,
    each_child: fn(&T) -> Vec<T>,
    view: fn(RwSignal<TreeNode<T, S>>) -> View,
    selection_model: RwSignal<SelectionModel<T, S>>,
    show_separator: FS,
    #[prop(optional, into)] selection_color: Option<Signal<String>>,
) -> impl IntoView
where
    T: Clone + PartialEq + 'static + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
    F: Fn() -> Vec<T> + Copy + 'static,
    FS: Fn() -> bool + Copy + 'static,
{
    let tree_nodes = move || {
        each()
            .iter()
            .map(|c| TreeNode::<T, S>::new(key, c.clone(), each_child, selection_model, 0))
            .collect::<Vec<_>>()
    };

    view! {
        <ul class="treeview">
        <For
            each=tree_nodes
            key=move |c| key(&c.row)
            children=move |item| {
                view! {
                    <TreeViewRow
                        node=item.clone()
                        selection_model=selection_model
                        view=view
                        each_child=each_child
                        selection_color
                    > {
                        view(create_rw_signal(item))
                    }</TreeViewRow>
                    <Show
                        when=show_separator
                        fallback=|| ()
                    >
                        <hr/>
                    </Show>
                }
            }
        />
        </ul>
    }
    .into_view()
}

#[component]
fn TreeViewRow<T, S>(
    children: Children,
    node: TreeNode<T, S>,
    each_child: fn(&T) -> Vec<T>,
    view: fn(RwSignal<TreeNode<T, S>>) -> View,
    selection_model: RwSignal<SelectionModel<T, S>>,
    selection_color: Option<Signal<String>>,
) -> impl IntoView
where
    T: Clone + PartialEq + 'static + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    let (key, _) = create_signal(node.get_key().clone());
    let node = create_read_slice(selection_model, move |sm| {
        sm.items.get(&key()).unwrap().clone()
    });

    let child_class = move || {
        let mut class = String::from("nested ");
        if node().is_expanded.get() {
            class += "active "
        }
        return class;
    };

    let caret_class = move || {
        if node().is_expanded.get() {
            "caret fa-solid fa-caret-right caret-down"
        } else {
            "caret fa-solid fa-caret-right"
        }
    };

    let div_class = move || {
        let mut class = String::from("selectable row");
        if selection_model().is_selected(&node().get_key()) {
            class += " selected"
        }

        return class;
    };

    let selection_style = move || {
        if selection_model().is_selected(&node().get_key()) {
            format!(
                "background: {};",
                selection_color
                    .map(|ac| ac())
                    .unwrap_or(String::from("#8BE9FD"))
            )
        } else {
            String::new()
        }
    };

    let on_click = move |_: MouseEvent| selection_model.update(|map| map.select(&node().get_key()));

    let on_caret_click = move |e: MouseEvent| {
        e.stop_propagation();
        node().toggle_expand()
    };

    let depth_style = move || {
        let margin = format!("{}em", 2.0 * node().depth as f64);
        let style = format!("padding-left:{};", margin);
        style
    };

    view! {
    <li>
        <div style={ move || { depth_style() + &selection_style() } } class=div_class on:click=on_click>
            <Show
                when= move || { each_child(&node.get_untracked().row).len() > 0 }
                fallback= move || {}
            >
                <span class=caret_class on:click=on_caret_click/>
            </Show>
            { children() }
        </div>
        <ul class=child_class>
        <For
            each=move || { node().children.get() }
            key=|c| c.get_key()
            children=move |item| {
                view! {
                    <TreeViewRow
                        node=item.clone()
                        selection_model=selection_model
                        each_child=each_child
                        view=view
                        selection_color
                    > {
                        view(create_rw_signal(item))
                    }</TreeViewRow>
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
    T: Clone + 'static + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub key: fn(&T) -> S,
    pub row: T,
    pub depth: usize,
    pub is_expanded: RwSignal<bool>,
    pub update: Trigger,
    pub children: RwSignal<Vec<TreeNode<T, S>>>,
    pub each_child: fn(&T) -> Vec<T>,
}

impl<T, S> TreeNode<T, S>
where
    T: Clone + 'static + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub fn new(
        key: fn(&T) -> S,
        item: T,
        each_child: fn(&T) -> Vec<T>,
        selection_model: RwSignal<SelectionModel<T, S>>,
        depth: usize,
    ) -> Self {
        let this = Self {
            key,
            row: item.clone(),
            depth,
            is_expanded: create_rw_signal(false),
            update: create_trigger(),
            children: create_rw_signal(Vec::new()),
            each_child,
        };

        selection_model.update(|map| {
            map.items.insert(key(&this.row), this.clone());
        });

        this.children.set(
            each_child(&item)
                .iter()
                .map(|c| TreeNode::new(key, c.clone(), each_child, selection_model, depth + 1))
                .collect(),
        );

        return this;
    }

    pub fn get_key(&self) -> S {
        (self.key)(&self.row)
    }

    pub fn insert_child(&self, item: T, selection_model: RwSignal<SelectionModel<T, S>>) {
        let node = TreeNode::new(
            self.key,
            item,
            self.each_child,
            selection_model,
            self.depth + 1,
        );
        self.children.update(|children| children.push(node))
    }

    pub fn set_expand(&self, do_expand: bool) {
        self.is_expanded.set(do_expand)
    }

    pub fn toggle_expand(&self) {
        self.is_expanded.set(!self.is_expanded.get())
    }
}