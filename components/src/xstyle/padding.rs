#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum XPadding {
    #[default]
    Default,
    Small,
    Medium,
    Big,
    Square(usize),
    Rect(usize, usize),
    Custom(usize, usize, usize, usize),
}

impl std::fmt::Display for XPadding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                XPadding::Default => "0".to_string(),
                XPadding::Small => "4px".to_string(),
                XPadding::Medium => "8px".to_string(),
                XPadding::Big => "16px".to_string(),
                XPadding::Square(square) => format!("{}px", square),
                XPadding::Rect(ypad, xpad) => format!("{}px {}px", ypad, xpad),
                XPadding::Custom(top, right, bottom, left) =>
                    format!("{}px {}px {}px {}px", top, right, bottom, left),
            }
        )
    }
}
