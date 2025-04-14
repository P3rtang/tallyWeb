use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XSize {
    Pixel(usize),
    Percentage(usize),
    Full,
}

impl std::fmt::Display for XSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                XSize::Pixel(px) => format!("{}px", px),
                XSize::Percentage(pc) => format!("{}%", pc),
                XSize::Full => "100%".to_string(),
            }
        )
    }
}

impl From<usize> for XSize {
    fn from(value: usize) -> Self {
        Self::Pixel(value)
    }
}
