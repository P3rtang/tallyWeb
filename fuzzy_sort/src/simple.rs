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
        for char in self.pattern.chars() {
            if let Some(i) = a[idx..].find(char) {
                score += 2 + (i == 0) as u32 * 5 + (idx == 0 && i == 0) as u32 * 5;
                idx += i + 1;
            } else if let Some(i) = a[idx..].to_lowercase().find(char.to_ascii_lowercase()) {
                score += 1 + (i == 0) as u32 * 5 + (idx == 0 && i == 0) as u32 * 5;
                idx += i + 1;
            }
            println!("char={char}, score={score}")
        }

        score
    }

    fn sort<T: Sortable + 'static>(&self) -> impl FnMut(&T, &T) -> std::cmp::Ordering {
        return move |a: &T, b: &T| self.score(b.as_str()).cmp(&self.score(a.as_str()));
    }
}
