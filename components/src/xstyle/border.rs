#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XBorderRadius {
    Full,
    Button,
    ButtonBig,
    Pixel(usize),
    Percentage(usize),
}

impl std::fmt::Display for XBorderRadius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                XBorderRadius::Full => "10000px".to_string(),
                XBorderRadius::Button => "8px".to_string(),
                XBorderRadius::ButtonBig => "12px".to_string(),
                XBorderRadius::Pixel(px) => format!("{}px", px),
                XBorderRadius::Percentage(per) => format!("{}%", per),
            }
        )
    }
}
