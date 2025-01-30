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
