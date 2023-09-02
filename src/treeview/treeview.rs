#![allow(unused_braces)]
#![allow(non_snake_case)]

use core::fmt::Debug;
use std::collections::HashMap;

use leptos::{ev::MouseEvent, *};

pub type SignalNodeChildren<T> = RwSignal<HashMap<T, TreeViewNode<T>>>;

/// TODO: wrap T in a smart pointer
pub trait TreeViewNodeItem<T: TreeViewNodeItem<T>>:
    PartialEq + Eq + core::hash::Hash + Clone + 'static + std::ops::Deref + Debug
{
    fn into_view(self, cx: Scope) -> leptos::View;
    fn get_children(&self) -> Vec<T>;
}

#[component]
pub fn TreeView<T, F, A, IV>(cx: Scope, start_nodes: F, after: A) -> impl IntoView
where
    T: TreeViewNodeItem<T>,
    F: Fn() -> Vec<T> + Copy + 'static,
    A: Fn(Scope) -> IV + 'static,
    IV: IntoView,
{
    let item_node_map = HashMap::<T, TreeViewNode<T>>::new();
    let item_node_signal = create_rw_signal(cx, item_node_map);
    provide_context(cx, item_node_signal);

    let tree_nodes = move || start_nodes().iter().map(|c| {
        TreeViewNode::new(cx, c.clone(), 0)
    }).collect::<Vec<_>>();

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
        {
        }
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
    pub is_expanded: bool,
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
            is_expanded: false,
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
        self.children
            .update(|children| children.push(TreeViewNode::new(cx, item, self.depth + 1)))
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

#[derive(Debug, Clone)]
pub struct Selection<T>(pub HashMap<T, bool>)
where
    T: TreeViewNodeItem<T>;

impl<T> Selection<T>
where
    T: TreeViewNodeItem<T>,
{
    pub fn new() -> Self {
        return Self(HashMap::new());
    }
    fn set_active(&mut self, node: T, is_active: bool) {
        self.0.clear();
        self.0.insert(node, is_active);
    }

    fn is_active(&self, node: &T) -> bool {
        return self.0.get(node).map(|b| *b).unwrap_or_default();
    }
}

#[component]
pub fn TreeViewRow<T, F>(
    cx: Scope,
    class: F,
    children: Children,
    item: TreeViewNode<T>,
) -> impl IntoView
where
    F: Fn() -> String + 'static,
    T: TreeViewNodeItem<T>,
{
    let node_children = expect_context::<SignalNodeChildren<T>>(cx);
    let selection = expect_context::<RwSignal<Selection<T>>>(cx);
    let (get_item, _) = create_signal(cx, item.row);

    let node = create_read_slice(cx, node_children, move |nodes| {
        nodes.get(&get_item()).cloned()
    });

    let (is_expanded, set_expanded) = create_slice(
        cx,
        node_children,
        move |nodes| {
            nodes
                .get(&get_item())
                .map(|n| n.is_expanded)
                .unwrap_or_default()
        },
        move |nodes, n| {
            nodes.get_mut(&get_item()).map(|node| node.is_expanded = n);
        },
    );

    let child_class = move || {
        let mut class = String::from("nested ");
        if is_expanded.get() {
            class += "active "
        }
        return class;
    };

    let caret_class = move || {
        if is_expanded.get() {
            "caret fas fa-caret-down"
        } else {
            "caret fas fa-caret-right"
        }
    };

    let on_click = move |_: MouseEvent| {
        selection
            .update(|sel| sel.set_active(get_item().clone(), !sel.is_active(&get_item().clone())));
    };

    let on_caret_click = move |e: MouseEvent| {
        e.stop_propagation();
        set_expanded.set(!is_expanded.get())
    };

    let depth_style = move || {
        let margin = format!(
            "{}em",
            1.8 * node().map(|n| n.depth).unwrap_or_default() as f64
        );
        let style = format!("padding-left:{}", margin);
        style
    };

    view! { cx,
    <li>
        <div style={ depth_style } class=class on:click=on_click>
            <Show when= move || { get_item.get().get_children().len() > 0 }
            fallback= move |_| {}
            ><span class=caret_class on:click=on_caret_click/></Show>
        { children(cx) }</div>
        <ul class=child_class>
        <For
            each=item.children
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
