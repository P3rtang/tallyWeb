mod simple;
pub use simple::*;

pub trait FuzzySort<'a> {
    fn score(&self, a: &'a str) -> u32;
    fn sort<T: Sortable + 'static>(&self) -> impl FnMut(&T, &T) -> std::cmp::Ordering;
}

pub trait Sortable {
    fn as_str(&self) -> &str;
}
