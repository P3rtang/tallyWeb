mod simple;
pub use simple::*;

pub trait FuzzySort<'a> {
    fn score(self: &Self, a: &'a str) -> u32;
    fn sort<T: Sortable + 'static>(self: &Self) -> impl FnMut(&T, &T) -> std::cmp::Ordering;
}

pub trait Sortable {
    fn as_str<'a>(self: &'a Self) -> &'a str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score() {
        let sorter = SimpleMatch::new("foo");
        assert_eq!(sorter.score("bar"), 0);
        assert_eq!(sorter.score("faa"), 7);
        assert_eq!(sorter.score("faf"), 7);

        let sorter = SimpleMatch::new("foooooooooo");
        assert_eq!(sorter.score("o"), 7);
    }
}
