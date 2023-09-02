#![allow(unused_braces)]

use core::fmt::Debug;
use std::collections::HashMap;

use leptos::{ev::MouseEvent, *};

pub type SignalNodeChildren<T> = RwSignal<HashMap<T, TreeViewNode<T>>>;

pub trait IntoNode<T: IntoNode<T>>:
    PartialEq + Eq + core::hash::Hash + Clone + 'static + std::ops::Deref + Debug
{
    fn into_node(self, cx: Scope, depth: usize) -> TreeViewNode<T>;
    fn into_view(self, cx: Scope) -> leptos::View;
    fn get_children(&self) -> Vec<T>;
}

#[derive(Debug, Clone)]
pub struct TreeView<T>
where
    T: IntoNode<T>,
{
    nodes: Vec<T>,
}

impl<T> TreeView<T>
where
    T: IntoNode<T>,
{
    pub fn new(cx: Scope, first_node: Vec<T>) -> Self {
        let selection = Selection::<T>::new();
        let selection_signal = create_rw_signal(cx, selection.clone());
        provide_context(cx, selection_signal);

        let item_node_map = HashMap::<T, TreeViewNode<T>>::new();
        let item_node_signal = create_rw_signal(cx, item_node_map);
        provide_context(cx, item_node_signal);

        first_node.iter().for_each(|i| {
            i.clone().into_node(cx, 0);
        });

        return Self { nodes: first_node };
    }
}

impl<T> IntoView for TreeView<T>
where
    T: IntoNode<T>,
{
    fn into_view(self, cx: leptos::Scope) -> leptos::View {
        view! { cx,
            <ul class="treeview">{
                self.nodes.into_iter().map(|n| n.into_view(cx)).collect_view(cx)
            }</ul>
        }
        .into_view(cx)
    }
}

#[derive(Debug, Clone)]
pub struct TreeViewNode<T>
where
    T: IntoNode<T>,
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
    T: IntoNode<T>,
{
    pub fn new(cx: Scope, item: T, depth: usize) -> Self {
        let ptr = std::ptr::addr_of!(*item) as *const usize;
        log!("{:?}", ptr as usize);
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
                .map(|c| c.clone().into_node(cx, depth + 1))
                .collect(),
        );

        return this;
    }
}

impl<T> std::hash::Hash for TreeViewNode<T>
where
    T: IntoNode<T>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&self.row, state);
    }
}

impl<T> PartialEq for TreeViewNode<T>
where
    T: IntoNode<T>,
{
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.row, &*other.row)
    }
}

impl<T> Eq for TreeViewNode<T> where T: IntoNode<T> {}

#[derive(Debug, Clone)]
pub struct Selection<T>(pub HashMap<T, bool>)
where
    T: IntoNode<T>;

impl<T> Selection<T>
where
    T: IntoNode<T>,
{
    fn new() -> Self {
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
    T: IntoNode<T>,
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
            1.2 * node().map(|n| n.depth).unwrap_or_default() as f64
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
