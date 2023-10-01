#![allow(unused_braces)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use leptos::{ev::MouseEvent, *};

#[derive(Debug, Clone)]
pub struct SelectionModel<T, S>
where
    T: Clone + 'static + std::ops::Deref + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    items: HashMap<S, TreeNode<T, S>>,
    selection: HashMap<S, bool>,
    pub selected: RwSignal<Vec<RwSignal<T>>>,
    accent_color: Option<Signal<String>>,
}

impl<T, S> SelectionModel<T, S>
where
    T: Clone + 'static + std::ops::Deref + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub fn new(accent_color: Option<Signal<String>>, selected: RwSignal<Vec<RwSignal<T>>>) -> Self {
        return Self {
            items: HashMap::new(),
            selection: HashMap::new(),
            selected,
            accent_color,
        };
    }

    pub fn select(&mut self, key: &S) {
        let current_value = self.selection.get(key).cloned().unwrap_or_default();
        self.selection.clear();
        self.selection.insert(key.clone(), !current_value);

        self.selected.update(|s| *s = self.selection())
    }

    fn selection(&self) -> Vec<RwSignal<T>> {
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

    pub fn is_selected(&self, key: &S) -> bool {
        return self.selection.get(key).cloned().unwrap_or_default();
    }

    pub fn is_empty(&self) -> bool {
        self.selection().is_empty()
    }
}

#[component]
pub fn TreeViewWidget<T, F, S, FS>(
    cx: Scope,
    each: F,
    key: fn(&T) -> S,
    each_child: fn(&T) -> Vec<T>,
    view: fn(Scope, RwSignal<TreeNode<T, S>>) -> View,
    selection_model: RwSignal<SelectionModel<T, S>>,
    show_separator: FS,
) -> impl IntoView
where
    T: Clone + 'static + std::ops::Deref + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
    F: Fn() -> Vec<T> + Copy + 'static,
    FS: Fn() -> bool + Copy + 'static,
{
    let tree_nodes = move || {
        each()
            .iter()
            .map(|c| TreeNode::<T, S>::new(cx, key, c.clone(), each_child, 0))
            .collect::<Vec<_>>()
    };

    view! { cx,
        <ul class="treeview">
        <For
            each=tree_nodes
            key=move |c| key(&c.row.get_untracked())
            view=move |cx, item| {
                view! { cx,
                    <TreeViewRow
                        node=item.clone()
                        selection_model=selection_model
                        view=view
                        each_child=each_child
                    > {
                        view(cx, create_rw_signal(cx, item))
                    }</TreeViewRow>
                    <Show
                        when=show_separator
                        fallback=|_| ()
                    >
                        <hr/>
                    </Show>
                }
            }
        />
        </ul>
    }
    .into_view(cx)
}

#[component]
fn TreeViewRow<T, S>(
    cx: Scope,
    children: Children,
    node: TreeNode<T, S>,
    each_child: fn(&T) -> Vec<T>,
    view: fn(Scope, RwSignal<TreeNode<T, S>>) -> View,
    selection_model: RwSignal<SelectionModel<T, S>>,
) -> impl IntoView
where
    T: Clone + 'static + std::ops::Deref + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    let (node, _) = create_signal(cx, node);

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
                selection_model()
                    .accent_color
                    .map(|ac| ac.get())
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

    view! { cx,
    <li>
        <div style={ move || { depth_style() + &selection_style() } } class=div_class on:click=on_click>
            <Show
                when= move || { each_child(&node.get_untracked().row.get_untracked()).len() > 0 }
                fallback= move |_| {}
            >
                <span class=caret_class on:click=on_caret_click/>
            </Show>
            { children(cx) }
        </div>
        <ul class=child_class>
        <For
            each=move || { node().children.get() }
            key=|c| c.get_key()
            view=move |cx, item| {
                view! { cx,
                    <TreeViewRow
                        node=item.clone()
                        selection_model=selection_model
                        each_child=each_child
                        view=view
                    > {
                        view(cx, create_rw_signal(cx, item))
                    }</TreeViewRow>
                }
            }
        />
        </ul>
    </li>
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode<T, S>
where
    T: Clone + 'static + std::ops::Deref + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub key: fn(&T) -> S,
    pub row: RwSignal<T>,
    pub depth: usize,
    pub is_expanded: RwSignal<bool>,
    pub update: Trigger,
    pub children: RwSignal<Vec<TreeNode<T, S>>>,
    pub each_child: fn(&T) -> Vec<T>,
}

impl<T, S> TreeNode<T, S>
where
    T: Clone + 'static + std::ops::Deref + Debug,
    S: Clone + PartialEq + Eq + Hash + 'static,
{
    pub fn new(
        cx: Scope,
        key: fn(&T) -> S,
        item: T,
        each_child: fn(&T) -> Vec<T>,
        depth: usize,
    ) -> Self {
        let this = Self {
            key,
            row: create_rw_signal(cx, item.clone()),
            depth,
            is_expanded: create_rw_signal(cx, false),
            update: create_trigger(cx),
            children: create_rw_signal(cx, Vec::new()),
            each_child,
        };

        let nodes = expect_context::<RwSignal<SelectionModel<T, S>>>(cx);
        nodes.update(|map| {
            map.items
                .insert(key(&this.row.get_untracked()), this.clone());
        });

        this.children.set(
            each_child(&item)
                .iter()
                .map(|c| TreeNode::new(cx, key, c.clone(), each_child, depth + 1))
                .collect(),
        );

        return this;
    }

    pub fn get_key(&self) -> S {
        (self.key)(&self.row.get_untracked())
    }

    pub fn insert_child(&self, cx: Scope, item: T) {
        let node = TreeNode::new(cx, self.key, item, self.each_child, self.depth + 1);
        self.children.update(|children| children.push(node))
    }

    pub fn set_expand(&self, do_expand: bool) {
        self.is_expanded.set(do_expand)
    }

    pub fn toggle_expand(&self) {
        self.is_expanded.set(!self.is_expanded.get())
    }
}
