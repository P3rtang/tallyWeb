use ev::EventDescriptor;

use super::*;

pub struct EventCallback<Event: EventDescriptor<EventType = T>, T>(
    Arc<dyn Fn(T) + Send + Sync + 'static>,
    std::marker::PhantomData<Event>,
);

impl<Event, T> EventCallback<Event, T>
where
    Event: EventDescriptor<EventType = T>,
{
    pub fn call(&self, ev: T) {
        (self.0)(ev)
    }
}

impl<Event, T> Default for EventCallback<Event, T>
where
    Event: EventDescriptor<EventType = T>,
{
    fn default() -> Self {
        Self(Arc::new(move |_| ()), std::marker::PhantomData::default())
    }
}

impl<F, Event, T> From<F> for EventCallback<Event, T>
where
    Event: EventDescriptor<EventType = T>,
    F: Fn(T) + Send + Sync + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(value), Default::default())
    }
}
