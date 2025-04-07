use super::*;

pub trait IntoAttributeFn<M: Marker> {
    fn into_attr_fn(self: Self) -> AttributeFn;
}

pub trait Marker {}

pub struct FnMarker;
impl Marker for FnMarker {}

pub struct AttrMarker;
impl Marker for AttrMarker {}

#[derive(Clone)]
pub struct AttributeFn(std::sync::Arc<dyn Fn() -> AnyAttribute + Send + Sync + 'static>);

impl AttributeFn {
    pub fn call(&self) -> AnyAttribute {
        (self.0)()
    }
}

impl Default for AttributeFn {
    fn default() -> Self {
        Self(std::sync::Arc::new(move || ().into_any_attr()))
    }
}

impl std::fmt::Debug for AttributeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AttributeFn: Fn() -> {:?}", self.call())
    }
}

impl<F, A> From<F> for AttributeFn
where
    F: Fn() -> A + Send + Sync + 'static,
    A: Attribute + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(move || value().into_any_attr()))
    }
}

impl<A> IntoAttributeFn<AttrMarker> for A
where
    A: Attribute + Clone + Send + Sync + 'static,
{
    fn into_attr_fn(self) -> AttributeFn {
        AttributeFn(Arc::new(move || self.clone().into_any_attr()))
    }
}

impl<F, A> IntoAttributeFn<FnMarker> for F
where
    F: Fn() -> A + Send + Sync + 'static,
    A: Attribute + 'static,
{
    fn into_attr_fn(self) -> AttributeFn {
        AttributeFn(Arc::new(move || self().into_any_attr()))
    }
}
