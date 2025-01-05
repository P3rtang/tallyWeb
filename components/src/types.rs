use leptos::{prelude::*, reactive::diagnostics::SpecialNonReactiveZone};

pub trait ComponentState: Clone + 'static {}

// #[derive(Clone)]
// pub struct ChildComponent<T: Clone> {
//     fragment: std::rc::Rc<dyn Fn(T) -> Fragment>,
//     state: T,
// }

// impl<T: ComponentState> From<(Box<dyn Fn(T) -> Fragment>, T)> for ChildComponent<T> {
//     fn from(value: (Box<dyn Fn(T) -> Fragment>, T)) -> Self {
//         Self {
//             fragment: std::rc::Rc::new(value.0),
//             state: value.1,
//         }
//     }
// }

// impl From<Box<dyn Fn() -> Fragment>> for ChildComponent<()> {
//     fn from(value: Box<dyn Fn() -> Fragment>) -> Self {
//         Self {
//             fragment: std::rc::Rc::new(move |_| value()),
//             state: (),
//         }
//     }
// }

// impl From<(Box<dyn Fn(()) -> Fragment>, ())> for ChildComponent<()> {
//     fn from(value: (Box<dyn Fn(()) -> Fragment>, ())) -> Self {
//         Self {
//             fragment: std::rc::Rc::new(value.0),
//             state: (),
//         }
//     }
// }

// impl<State: ComponentState> IntoView for ChildComponent<State> {
//     fn into_view(self) -> View<Self> {
//         #[allow(unused_braces)]
//         view! { {(self.fragment)(self.state)} }.into()
//     }
// }

// impl IntoView for ChildComponent<()> {
//     fn into_view(self) -> View {
//         #[allow(unused_braces)]
//         view! { {(self.fragment)(())} }.into()
//     }
// }

pub trait FromClosure<T> {
    type Output;

    fn from_closure(closure: impl Fn(T) -> Self::Output + 'static) -> Self;
}

pub trait FromEmptyClosure {
    type Output;

    fn from_closure(closure: impl Fn() -> Self::Output + 'static) -> Self;
}

#[derive(Clone)]
pub enum Prop<T: Clone + Send + Sync> {
    Fn(std::sync::Arc<dyn Fn() -> T>),
    Value(T),
}

unsafe impl<T: Clone + Send + Sync> Send for Prop<T> {}
unsafe impl<T: Clone + Send + Sync> Sync for Prop<T> {}

impl<T: Clone + Send + Sync> From<T> for Prop<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: Clone + Send + Sync> FromEmptyClosure for Prop<T> {
    type Output = T;
    fn from_closure(closure: impl Fn() -> Self::Output + 'static) -> Self {
        Self::Fn(std::sync::Arc::new(closure))
    }
}

macro_rules! attr_signal_type {
    ($signal_type:ty) => {
        impl<T: Clone + Sync + Send + 'static> From<$signal_type> for Prop<T> {
            fn from(value: $signal_type) -> Self {
                SpecialNonReactiveZone::enter();
                let modified_fn = std::sync::Arc::new(move || value.get_untracked());
                Self::Fn(modified_fn)
            }
        }
    };
}

attr_signal_type!(ReadSignal<T>);
attr_signal_type!(RwSignal<T>);
attr_signal_type!(Memo<T>);
attr_signal_type!(Signal<T>);

impl<T: Clone + Send + Sync> From<Box<dyn Fn() -> T>> for Prop<T> {
    fn from(value: Box<dyn Fn() -> T>) -> Self {
        Self::Fn(value.into())
    }
}

impl<T: Clone + Send + Sync> From<std::sync::Arc<dyn Fn() -> T>> for Prop<T> {
    fn from(value: std::sync::Arc<dyn Fn() -> T>) -> Self {
        Self::Fn(value)
    }
}

impl<T: Clone + Send + Sync + 'static> From<fn() -> T> for Prop<T> {
    fn from(value: fn() -> T) -> Self {
        Self::Fn(std::sync::Arc::new(value))
    }
}

impl<T: Clone + Send + Sync> std::ops::FnOnce<()> for Prop<T> {
    type Output = T;

    extern "rust-call" fn call_once(self, _: ()) -> Self::Output {
        match self {
            Prop::Fn(rc) => rc(),
            Prop::Value(b) => b,
        }
    }
}

impl<T: Clone + Send + Sync> std::ops::FnMut<()> for Prop<T> {
    extern "rust-call" fn call_mut(&mut self, _: ()) -> Self::Output {
        match self {
            Prop::Fn(rc) => rc(),
            Prop::Value(b) => b.clone(),
        }
    }
}

impl<T: Clone + Send + Sync> std::ops::Fn<()> for Prop<T> {
    extern "rust-call" fn call(&self, _: ()) -> Self::Output {
        match self {
            Prop::Fn(rc) => rc(),
            Prop::Value(b) => b.clone(),
        }
    }
}
