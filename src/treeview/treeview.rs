#![allow(unused_braces)]

use std::{cell::RefCell, rc::Rc, collections::HashMap, ptr};
use core::fmt::Debug;

use leptos::{*};

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub trait IntoNode<T: IntoNode<T>>: Clone {
    fn into_node(self, cx: Scope, depth: usize) -> TreeViewNode<T>;
    fn into_view(self, cx: Scope) -> leptos::View;
    fn get_children(&self) -> Vec<T>;
}

#[derive(Debug, Clone)]
pub struct RcTreeViewNode<T: IntoNode<T>>(
    Rc<RefCell<TreeViewNode<T>>>,
    RwSignal<bool>,
);

impl<T: IntoNode<T>>  RcTreeViewNode<T> {
    fn from_node(cx: Scope, value: TreeViewNode<T>) -> Self {
        let show_signal = create_rw_signal(cx, false);
        let this = Self(Rc::new(RefCell::new(value)), show_signal);
        return this
    }

    pub fn item(&self) -> T {
        return self.0.borrow_mut().row.clone()
    }
}

impl<T> IntoView for RcTreeViewNode<T>
where T: IntoNode<T> + 'static {
    fn into_view(self, cx: leptos::Scope) -> leptos::View {
        let selection = expect_context::<RwSignal<Selection<T>>>(cx);
        let clone = self.clone();

        let div_class = enclose!((clone) move || {
            let mut class = String::from("selectable row ");
            if selection.with(|sel| sel.is_active(&clone)) {
                class += "selected"
            }

            return class
        });

        let child_class = enclose!((clone) move || {
            let mut class = String::from("nested ");
            if clone.1.get() {
                class += "active "
            }
            return class
        });

        let caret_class = enclose!((clone) move || {
            if clone.1.get() {
                "caret fas fa-caret-down"
            } else {
                "caret fas fa-caret-right"
            }
        });

        let on_click = enclose!((clone) move |_| {
            selection.update(|sel| sel.set_active(clone.clone(), !sel.is_active(&clone)));
        });

        let on_caret_click = enclose!((clone) move |_| {
            clone.1.set(!clone.1.get())
        });

        let create_indent = enclose!((clone) move || {
            (0..clone.0.borrow().depth).into_iter().map(|_| view! {cx, <span class="indent"/>}).collect_view(cx)
        });

        view! { cx,
            <li>
                <div class=div_class> {
                        if clone.0.borrow().nodes.len() > 0 {
                            view! { cx,
                                { create_indent() }
                                <span class=caret_class on:click=on_caret_click/>
                            }.into_view(cx)
                        } else {
                            view! { cx, { create_indent() } }
                        }
                    }
                    <span on:click=on_click >
                        { enclose!((clone) move || clone.0.borrow_mut().clone().row.into_view(cx)) }
                    </span>
                </div>
                <ul class=child_class>{ move || clone.0.borrow_mut().nodes.clone() }</ul>
            </li>
        }.into_view(cx)
    }
}

impl<T> core::hash::Hash for RcTreeViewNode<T>
where T: IntoNode<T> + 'static {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(&*self.0, state)
    }
}

impl<T> PartialEq for RcTreeViewNode<T>
where T: IntoNode<T> + 'static {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(&*self.0, &*other.0)
    }
}

impl<T> Eq for RcTreeViewNode<T>
where T: IntoNode<T> + 'static {}

pub struct TreeView<T> where T: IntoNode<T> + 'static {
    pub selection_signal: RwSignal<Selection<T>>,

    nodes: Vec<RcTreeViewNode<T>>,
}

impl<T> TreeView<T> where T: IntoNode<T> {
    pub fn new(cx: Scope) -> Self {
        let selection = Selection(HashMap::new());
        let selection_signal = create_rw_signal(cx, selection.clone());
        provide_context(cx, selection_signal);
        return Self { selection_signal, nodes: Vec::new() }
    }

    pub fn set_node(&mut self, cx: Scope, nodes: Vec<TreeViewNode<T>>) {
        let nodes = nodes.iter().map(|n| {
            RcTreeViewNode::from_node(cx, n.clone())
        }).collect::<Vec<_>>();


        self.nodes = nodes
    }
}

impl<T> IntoView for TreeView<T> where T: IntoNode<T> + 'static {
    fn into_view(self, cx: leptos::Scope) -> leptos::View {
        view! { cx,
            <ul class="treeview">
            { self.nodes.into_iter().map(|rc_n| rc_n.into_view(cx)).collect_view(cx) }
            </ul>
        }.into_view(cx)
    }
}

#[derive(Debug, Clone)]
pub struct TreeViewNode<T> where T: IntoNode<T> {
    row: T,
    nodes: Vec<RcTreeViewNode<T>>,
    depth: usize,
}

impl<T> TreeViewNode<T> where T: IntoNode<T> + 'static {
    pub fn new(cx: Scope, depth: usize, item: T) -> Self {
        let nodes = item.get_children().into_iter().map(|c| {
            RcTreeViewNode::from_node(cx, c.into_node(cx, depth + 1))
        }).collect::<Vec<_>>();
        return Self { row: item, nodes, depth }
    }
}

#[derive(Debug, Clone)]
pub struct Selection<T>(pub HashMap<RcTreeViewNode<T>, bool>)
     where T: IntoNode<T> + 'static;

impl<T> Selection<T>
where T: IntoNode<T> {
    fn set_active(&mut self, node: RcTreeViewNode<T>, is_active: bool) {
        self.0.clear();
        self.0.insert(node, is_active);
    }

    fn is_active(&self, node: &RcTreeViewNode<T>) -> bool {
        return self.0.get(node).map(|b| *b).unwrap_or_default()
    }
}

#[component]
fn TreeViewRow(cx: Scope, class: String) -> impl IntoView {

}
