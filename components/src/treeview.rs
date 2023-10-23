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

    pub fn select(&mut self, key: &S) {
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
            .map(|(k, _)| &self.items.get(k).unwrap().row)
            .collect::<Vec<_>>()
    }

    pub fn get_selected_keys(&self) -> Vec<&S> {
        self.selection
            .iter()
            .filter(|(_, b)| **b)
            .map(|(key, _)| key)
            .collect()
    }

    pub fn remove_item(&mut self, key: &S) -> Option<T> {
        Some(self.items.remove(key)?.row)
    }

    pub fn is_selected(&self, key: &S) -> bool {
        return self.selection.get(key).cloned().unwrap_or_default();
    }

    pub fn is_empty(&self) -> bool {
        self.selection().is_empty()
    }
}

#[component]
pub fn TreeViewWidget<T, F, S, FV, IV>(
    each: F,
    key: fn(&T) -> S,
    each_child: fn(&T) -> Vec<T>,
    view: FV,
    #[prop(default=create_signal(false).0.into(), into)] show_separator: Signal<bool>,
    #[prop(default=create_rw_signal(SelectionModel::default()), into)] selection_model: RwSignal<
        SelectionModel<S, T>,
    >,
    #[prop(optional, into)] selection_color: Option<Signal<String>>,
) -> impl IntoView
where
    T: Clone + PartialEq + 'static + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
    F: Fn() -> Vec<T> + Copy + 'static,
    FV: Fn(S) -> IV + Copy + 'static,
    IV: IntoView,
{
    let tree_nodes = move || {
        each()
            .into_iter()
            .map(|c| TreeNode::<T, S>::new(key, c, each_child, selection_model, 0))
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
                        key=item.get_key()
                        selection_model=selection_model
                        view=view
                        each_child=each_child
                        selection_color
                    > {
                        view(item.get_key())
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
fn TreeViewRow<T, S, FV, IV>(
    children: ChildrenFn,
    key: S,
    each_child: fn(&T) -> Vec<T>,
    view: FV,
    selection_model: RwSignal<SelectionModel<S, T>>,
    selection_color: Option<Signal<String>>,
) -> impl IntoView
where
    T: Clone + PartialEq + 'static + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
    FV: Fn(S) -> IV + Copy + 'static,
    IV: IntoView,
{
    let (key, _) = create_signal(key);

    let node = create_read_slice(selection_model, move |sm| sm.items.get(&key()).cloned());

    let (is_expanded, toggle_expand) = create_slice(
        selection_model,
        move |model| {
            model
                .items
                .get(&key())
                .map(|n| n.is_expanded)
                .unwrap_or_default()
        },
        move |model, _| {
            if let Some(node) = model.items.get_mut(&key()) {
                node.toggle_expand()
            };
        },
    );

    let (is_selected, set_selected) = create_slice(
        selection_model,
        move |model| model.is_selected(&key()),
        move |model, _| model.select(&key()),
    );

    let child_class = move || {
        let mut class = String::from("nested ");
        if is_expanded() {
            class += "active "
        }
        class
    };

    let caret_class = move || {
        if is_expanded() {
            "caret fa-solid fa-caret-right caret-down"
        } else {
            "caret fa-solid fa-caret-right"
        }
    };

    let div_class = move || {
        let mut class = String::from("selectable row");
        if is_selected() {
            class += " selected"
        }

        class
    };

    let selection_style = move || {
        if is_selected() {
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

    let on_click = move |_: MouseEvent| set_selected(());

    let on_caret_click = move |e: MouseEvent| {
        e.stop_propagation();
        toggle_expand(())
    };

    let depth_style = move || {
        let margin = format!(
            "{}em",
            2.0 * node().map(|n| n.depth).unwrap_or_default() as f64
        );
        let style = format!("padding-left:{};", margin);
        style
    };

    let node_children = create_read_slice(selection_model, move |model| {
        node()
            .map(|n| n.get_node_children(model))
            .unwrap_or_default()
    });

    view! {
    <Show
        when=move || node().is_some()
    >
    <li>
        <div style={ move || { depth_style() + &selection_style() } } class=div_class on:click=on_click>
            <Show
                when= move || { !each_child(&node.get_untracked().unwrap().row).is_empty() }
                fallback= move || {}
            >
                <span class=caret_class on:click=on_caret_click/>
            </Show>
            { children() }
        </div>
        <ul class=child_class>
        <For
            each=node_children
            key=|c| c.get_key()
            children=move |item| {
                view! {
                    <TreeViewRow
                        key=item.get_key()
                        selection_model=selection_model
                        each_child=each_child
                        view=view
                        selection_color
                    > {
                        view(item.get_key())
                    }</TreeViewRow>
                }
            }
        />
        </ul>
    </li>
    </Show>
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
    pub each_child: fn(&T) -> Vec<T>,
}

impl<T, S> TreeNode<T, S>
where
    T: Clone + 'static + Debug + PartialEq,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub fn new(
        key: fn(&T) -> S,
        item: T,
        each_child: fn(&T) -> Vec<T>,
        selection_model: RwSignal<SelectionModel<S, T>>,
        depth: usize,
    ) -> Self {
        let this = Self {
            key,
            row: item.clone(),
            depth,
            is_expanded: false,
            each_child,
        };

        selection_model.update(|map| {
            map.items.insert(key(&this.row), this.clone());
        });

        each_child(&item).iter().for_each(|c| {
            TreeNode::new(key, c.clone(), each_child, selection_model, depth + 1);
        });

        this
    }

    pub fn get_key(&self) -> S {
        (self.key)(&self.row)
    }

    pub fn insert_child(&self, item: T, selection_model: &mut SelectionModel<S, T>) {
        let node = TreeNode {
            key: self.key,
            row: item.clone(),
            depth: self.depth + 1,
            is_expanded: false,
            each_child: self.each_child,
        };

        selection_model.items.insert((self.key)(&item), node);
    }

    pub fn get_children_keys(&self) -> Vec<S> {
        (self.each_child)(&self.row)
            .iter()
            .map(|i| (self.key)(i))
            .collect()
    }

    pub fn get_node_children(&self, model: &SelectionModel<S, T>) -> Vec<TreeNode<T, S>> {
        self.get_children_keys()
            .iter()
            .filter_map(|key| model.items.get(key).cloned())
            .collect::<Vec<_>>()
    }

    pub fn set_expand(&mut self, do_expand: bool) {
        self.is_expanded = do_expand
    }

    pub fn toggle_expand(&mut self) {
        self.is_expanded = !self.is_expanded
    }
}
