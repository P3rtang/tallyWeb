use ev::EventDescriptor;

use super::*;

#[derive(Clone)]
pub struct EventCallback<T>(Arc<dyn Fn(T) + Send + Sync + 'static>);

impl<T> EventCallback<T> {
    pub fn call(&self, ev: T) {
        (self.0)(ev)
    }
}

impl<T> Default for EventCallback<T> {
    fn default() -> Self {
        Self(Arc::new(move |_| ()))
    }
}

impl<F, T> From<F> for EventCallback<T>
where
    F: Fn(T) + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> FnOnce<(T,)> for EventCallback<T> {
    type Output = ();

    extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
        self.call(args.0);
    }
}

impl<T> FnMut<(T,)> for EventCallback<T> {
    extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
        self.call(args.0)
    }
}

#[derive(Clone)]
pub struct ChildrenPropFn<T>(Arc<dyn Fn(T) -> AnyView + Send + Sync + 'static>)
where
    T: Clone + 'static;

impl<T, F> From<F> for ChildrenPropFn<T>
where
    F: Fn(T) -> AnyView + Send + Sync + 'static,
    T: Clone + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> FnOnce<(T,)> for ChildrenPropFn<T>
where
    T: Clone + 'static,
{
    type Output = AnyView;

    extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
        (self.0)(args.0)
    }
}

#[derive(Clone)]
pub struct TypedChildrenPropFn<T, IV>(Arc<dyn Fn(T) -> View<IV> + Send + Sync + 'static>)
where
    T: Clone + 'static,
    IV: Sized;

impl<T, F, IV> From<F> for TypedChildrenPropFn<T, IV>
where
    F: Fn(T) -> View<IV> + Send + Sync + 'static,
    T: Clone + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

impl<T, IV> FnOnce<(T,)> for TypedChildrenPropFn<T, IV>
where
    T: Clone + 'static,
{
    type Output = View<IV>;

    extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
        (self.0)(args.0)
    }
}

impl<F, T, IV> ToChildren<F> for TypedChildrenPropFn<T, IV>
where
    F: Fn(T) -> IV + Send + Sync + 'static,
    IV: IntoView,
    IV::AsyncOutput: Send,
    T: Clone + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Self(Arc::new(move |c| f(c).into_view()))
    }
}

impl<F, T, IV> ToChildren<ChildrenOptContainer<(F, T)>> for TypedChildrenPropFn<T, IV>
where
    F: Fn(T) -> IV + Send + Sync + 'static,
    IV: IntoView,
    IV::AsyncOutput: Send,
    T: Clone + 'static,
{
    #[inline]
    fn to_children(f: ChildrenOptContainer<(F, T)>) -> Self {
        let (f, t) = f.0;
        Self(Arc::new(move |c| f(c).into_view()))
    }
}
