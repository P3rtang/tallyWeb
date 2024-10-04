use super::*;

pub struct SimpleMatch {
    pattern: String,
}

impl SimpleMatch {
    pub fn new(pattern: impl ToString) -> Self {
        Self {
            pattern: pattern.to_string(),
        }
    }

    pub fn set(&mut self, pattern: impl ToString) {
        self.pattern = pattern.to_string()
    }
}

impl<'a> FuzzySort<'a> for SimpleMatch {
    fn score(&self, a: &'a str) -> u32 {
        let mut idx = 0;
        let mut score = 0;
        for (pat_idx, char) in self.pattern.chars().enumerate() {
            if let Some(i) = a[idx..].find(char) {
                score += 2 + (i == 0) as u32 * 2 + (pat_idx == 0) as u32 * 2;
                idx += i + 1;
            } else if let Some(i) = a[idx..].to_lowercase().find(char.to_ascii_lowercase()) {
                score += 1 + (i == 0) as u32 * 2 + (pat_idx == 0) as u32 * 2;
                idx += i + 1;
            }
            println!("char={char}, score={score}")
        }

        score
    }

    fn sort<T: Sortable + 'static>(&self) -> impl FnMut(&T, &T) -> std::cmp::Ordering {
        move |a: &T, b: &T| self.score(b.as_str()).cmp(&self.score(a.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_score() {
        let sorter = SimpleMatch::new("foo");
        assert_eq!(sorter.score("bar"), 0);
        assert_eq!(sorter.score("faa"), 6);
        assert_eq!(sorter.score("afa"), 4);
        assert_eq!(sorter.score("faf"), 6);
        assert_eq!(sorter.score("ko"), 2);
    }
}
