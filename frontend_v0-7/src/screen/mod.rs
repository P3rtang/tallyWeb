use super::*;

mod client;
pub mod server;

// TODO: factor the types out to a new file
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Screen {
    pub height: f64,
    pub width: f64,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            height: 1080.0,
            width: 1920.0,
        }
    }
}

impl Screen {
    pub fn viewport(&self) -> ViewPort {
        match self.width {
            w if w > 1563.0 => ViewPort::XLarge,
            w if w > 1200.0 => ViewPort::Large,
            w if w > 900.0 => ViewPort::Medium,
            w if w > 600.0 => ViewPort::Small,
            _ => ViewPort::XSmall,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ViewPort {
    XSmall,
    Small,
    Medium,
    Large,
    XLarge,
}
