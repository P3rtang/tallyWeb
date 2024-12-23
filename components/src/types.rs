use leptos::*;

pub trait ComponentState: Clone + 'static {}

#[derive(Clone)]
pub struct ChildComponent<T: Clone> {
    fragment: std::rc::Rc<dyn Fn(T) -> Fragment>,
    state: T,
}

impl<T: ComponentState> From<(Box<dyn Fn(T) -> Fragment>, T)> for ChildComponent<T> {
    fn from(value: (Box<dyn Fn(T) -> Fragment>, T)) -> Self {
        Self {
            fragment: std::rc::Rc::new(value.0),
            state: value.1,
        }
    }
}

impl From<Box<dyn Fn() -> Fragment>> for ChildComponent<()> {
    fn from(value: Box<dyn Fn() -> Fragment>) -> Self {
        Self {
            fragment: std::rc::Rc::new(move |_| value()),
            state: (),
        }
    }
}

impl From<(Box<dyn Fn(()) -> Fragment>, ())> for ChildComponent<()> {
    fn from(value: (Box<dyn Fn(()) -> Fragment>, ())) -> Self {
        Self {
            fragment: std::rc::Rc::new(value.0),
            state: (),
        }
    }
}

impl<T: ComponentState> IntoView for ChildComponent<T> {
    fn into_view(self) -> View {
        #[allow(unused_braces)]
        view! { {(self.fragment)(self.state)} }.into()
    }
}

impl IntoView for ChildComponent<()> {
    fn into_view(self) -> View {
        #[allow(unused_braces)]
        view! { {(self.fragment)(())} }.into()
    }
}

pub trait FromClosure<T> {
    type Output;

    fn from_closure(closure: impl Fn(T) -> Self::Output + 'static) -> Self;
}

pub trait FromEmptyClosure {
    type Output;

    fn from_closure(closure: impl Fn() -> Self::Output + 'static) -> Self;
}

#[derive(Clone)]
pub enum Prop<T: Clone> {
    Fn(std::rc::Rc<dyn Fn() -> T>),
    Value(T),
}

impl<T: Clone> From<T> for Prop<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: Clone> FromEmptyClosure for Prop<T> {
    type Output = T;
    fn from_closure(closure: impl Fn() -> Self::Output + 'static) -> Self {
        Self::Fn(std::rc::Rc::new(closure))
    }
}

macro_rules! attr_signal_type {
    ($signal_type:ty) => {
        impl<T: Clone> From<$signal_type> for Prop<T> {
            fn from(value: $signal_type) -> Self {
                let modified_fn = std::rc::Rc::new(move || value.get());
                Self::Fn(modified_fn)
            }
        }
    };
}

attr_signal_type!(ReadSignal<T>);
attr_signal_type!(RwSignal<T>);
attr_signal_type!(Memo<T>);
attr_signal_type!(Signal<T>);
attr_signal_type!(MaybeSignal<T>);

impl<T: Clone> From<Box<dyn Fn() -> T>> for Prop<T> {
    fn from(value: Box<dyn Fn() -> T>) -> Self {
        Self::Fn(value.into())
    }
}

impl<T: Clone> From<std::rc::Rc<dyn Fn() -> T>> for Prop<T> {
    fn from(value: std::rc::Rc<dyn Fn() -> T>) -> Self {
        Self::Fn(value)
    }
}

impl<T: Clone + 'static> From<fn() -> T> for Prop<T> {
    fn from(value: fn() -> T) -> Self {
        Self::Fn(std::rc::Rc::new(value))
    }
}

impl<T: Clone> std::ops::FnOnce<()> for Prop<T> {
    type Output = T;

    extern "rust-call" fn call_once(self, _: ()) -> Self::Output {
        match self {
            Prop::Fn(rc) => rc(),
            Prop::Value(b) => b,
        }
    }
}

impl<T: Clone> std::ops::FnMut<()> for Prop<T> {
    extern "rust-call" fn call_mut(&mut self, _: ()) -> Self::Output {
        match self {
            Prop::Fn(rc) => rc(),
            Prop::Value(b) => b.clone(),
        }
    }
}

impl<T: Clone> std::ops::Fn<()> for Prop<T> {
    extern "rust-call" fn call(&self, _: ()) -> Self::Output {
        match self {
            Prop::Fn(rc) => rc(),
            Prop::Value(b) => b.clone(),
        }
    }
}
