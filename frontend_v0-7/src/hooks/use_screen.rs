use super::*;

// TODO: factor the types out to a new file
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Screen {
    pub height: f64,
    pub width: f64,
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

pub fn use_screen() -> Memo<Screen> {
    let (screen, set_screen) = signal(Screen::default());

    #[cfg(not(feature = "ssr"))]
    {
        let win = window();

        request_animation_frame(move || {
            set_screen.update(|s| {
                s.width = win
                    .inner_width()
                    .ok()
                    .and_then(|w| w.as_f64())
                    .unwrap_or(s.width);
                s.height = win
                    .inner_height()
                    .ok()
                    .and_then(|w| w.as_f64())
                    .unwrap_or(s.width);
            })
        });

        let win = window();

        window_event_listener(leptos::ev::resize, move |_| {
            set_screen.update(|s| {
                s.width = win
                    .inner_width()
                    .ok()
                    .and_then(|w| w.as_f64())
                    .unwrap_or(s.width);
                s.height = win
                    .inner_height()
                    .ok()
                    .and_then(|w| w.as_f64())
                    .unwrap_or(s.width);
            })
        });
    }

    Memo::new(move |_| screen.get())
}
