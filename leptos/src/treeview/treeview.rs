#![allow(unused_braces)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::fmt::Debug;
use std::{collections::HashMap, ops::Deref};

use leptos::{ev::MouseEvent, *};

pub type SignalNodeChildren<T> = RwSignal<PointerMap<T, TreeViewNode<T>>>;
pub type Selection<T> = RwSignal<PointerMap<T, bool>>;

#[derive(Debug, Clone)]
pub struct PointerMap<K, V>(HashMap<HashWrapper<K>, V>)
where K: Deref + Clone;

impl<K, V> PointerMap<K, V>
where K: Deref + Clone,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        return self.0.get(&HashWrapper(key.clone()))
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.0.insert(HashWrapper(key), value);
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl<'a, K, V> IntoIterator for &'a PointerMap<K, V>
where K: Deref + Clone {
    type Item = (&'a K, &'a V);

    type IntoIter = std::vec::IntoIter<(&'a K, &'a V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(|(w, v)| (w.inner(), v)).collect::<Vec<_>>().into_iter()
    }
}

impl<K, V> IntoIterator for PointerMap<K, V>
where K: Deref + Clone
{
    type Item = (K, V);

    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|(w, v)| (w.into_inner(), v)).collect::<Vec<_>>().into_iter()
    }
}

#[derive(Debug, Clone)]
struct HashWrapper<T>(T)
where T: Deref + Clone;

impl<T: Deref + Clone> HashWrapper<T> {
    pub fn inner(&self) -> &T {
        return &self.0
    }
    pub fn into_inner(self) -> T {
        return self.0
    }
}

impl<T: Deref + Clone> PartialEq for HashWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        let ptr = std::ptr::addr_of!(*self.0) as *const usize;
        let other_ptr = std::ptr::addr_of!(*other.0) as *const usize;
        ptr == other_ptr
    }
}

impl<T: Deref + Clone> Eq for HashWrapper<T> {}

impl<T: Deref + Clone> core::hash::Hash for HashWrapper<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let ptr = std::ptr::addr_of!(*self.0) as *const usize;
        ptr.hash(state)
    }
}

/// TODO: wrap T in a smart pointer
pub trait TreeViewNodeItem<T: TreeViewNodeItem<T>>:
    Clone + 'static + std::ops::Deref + Debug
{
    fn into_view(self, cx: Scope) -> leptos::View;
    fn get_children(&self) -> Vec<T>;
}

#[component]
pub fn TreeViewWidget<T, F, A, IV>(
    cx: Scope,
    start_nodes: F,
    after: A,
) -> impl IntoView
where
    T: TreeViewNodeItem<T>,
    F: Fn() -> Vec<T> + Copy + 'static,
    A: Fn(Scope) -> IV + 'static,
    IV: IntoView,
{
    let item_node_map: PointerMap<T, TreeViewNode<T>> = PointerMap::new();
    let item_node_signal = create_rw_signal(cx, item_node_map);
    provide_context(cx, item_node_signal);

    let tree_nodes = move || {
        start_nodes()
            .iter()
            .map(|c| TreeViewNode::new(cx, c.clone(), 0))
            .collect::<Vec<_>>()
    };

    view! { cx,
        <ul class="treeview">
        <For
            each=tree_nodes
            key=|c| c.id
            view=move |cx, item| {
                view! { cx, {
                    item.row.clone().into_view(cx)
                }}
            }
        />
        <li>{ after(cx) }</li>
        </ul>
    }
    .into_view(cx)
}

#[derive(Debug, Clone)]
pub struct TreeViewNode<T>
where
    T: TreeViewNodeItem<T>,
{
    pub id: usize,
    pub row: T,
    pub depth: usize,
    pub is_expanded: RwSignal<bool>,
    pub update: Trigger,
    pub children: RwSignal<Vec<TreeViewNode<T>>>,
}

impl<T> TreeViewNode<T>
where
    T: TreeViewNodeItem<T>,
{
    pub fn new(cx: Scope, item: T, depth: usize) -> Self {
        let ptr = std::ptr::addr_of!(*item) as *const usize;
        let this = Self {
            id: ptr as usize,
            row: item.clone(),
            depth,
            is_expanded: create_rw_signal(cx, false),
            update: create_trigger(cx),
            children: create_rw_signal(cx, Vec::new()),
        };

        let item_node_signal = expect_context::<SignalNodeChildren<T>>(cx);
        item_node_signal.update(|map| {
            map.insert(item.clone(), this.clone());
        });

        this.children.set(
            item.get_children()
                .iter()
                .map(|c| TreeViewNode::new(cx, c.clone(), depth + 1))
                .collect(),
        );

        return this;
    }

    pub fn insert_child(&self, cx: Scope, item: T) {
        let node = TreeViewNode::new(cx, item, self.depth + 1);
        self.children.update(|children| children.push(node))
    }

    pub fn set_expand(&self, do_expand: bool) {
        self.is_expanded.set(do_expand)
    }

    pub fn toggle_expand(&self) {
        self.is_expanded.set(!self.is_expanded.get())
    }
}

impl<T> std::hash::Hash for TreeViewNode<T>
where
    T: TreeViewNodeItem<T>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&self.row, state);
    }
}

impl<T> PartialEq for TreeViewNode<T>
where
    T: TreeViewNodeItem<T>,
{
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.row, &*other.row)
    }
}

impl<T> Eq for TreeViewNode<T> where T: TreeViewNodeItem<T> {}

#[component]
pub fn TreeViewRow<T>(
    cx: Scope,
    children: Children,
    node: TreeViewNode<T>,

    #[prop(optional)]
    selection: Option<Selection<T>>,
) -> impl IntoView
where
    T: TreeViewNodeItem<T>,
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
            "caret fas fa-caret-down"
        } else {
            "caret fas fa-caret-right"
        }
    };


    let div_class = move || {
        let mut class = String::from("selectable row");
        if selection.map(|sel| sel.get().get(&node().row).cloned()).flatten().unwrap_or_default() {
            class += " selected"
        }

        return class;
    };

    let on_click = move |_: MouseEvent| {
        selection.map(|sel| sel.update(|map| {
            let current_value = map.get(&node().row).cloned().unwrap_or_default();
            map.clear();
            map.insert(node().row, !current_value)
        }));
    };

    let on_caret_click = move |e: MouseEvent| {
        e.stop_propagation();
        node().toggle_expand()
    };

    let depth_style = move || {
        let margin = format!("{}em", 2.0 * node().depth as f64);
        let style = format!("padding-left:{}", margin);
        style
    };

    view! { cx,
    <li>
        <div style={ depth_style } class=div_class on:click=on_click>
            <Show when= move || { node().row.get_children().len() > 0 }
            fallback= move |_| {}
            ><span class=caret_class on:click=on_caret_click/></Show>
        { children(cx) }</div>
        <ul class=child_class>
        <For
            each=move || { node().children.get() }
            key=|c| c.id
            view=move |cx, item| {
                view! { cx, {
                    item.row.clone().into_view(cx)
                }}
            }
        />
        </ul>
    </li>
    }
}
