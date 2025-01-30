use super::*;

pub fn use_screen() -> Memo<Screen> {
    let screen_rsc = use_context::<Resource<Screen>>().ok_or(AppError::MissingScreenSignal("use_screen".to_string(), String::new())).unwrap();

    #[cfg(not(feature = "ssr"))]
    {
        Effect::new(move |prev| {
            let screen = screen_rsc.get().unwrap_or_default();

            if Some(screen) != prev {
                screen.set_cookie();
            }

            screen
        });

        let win = window();

        request_animation_frame(move || {
            screen_rsc.update(|s| {
                if let Some(s) = s {
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
                }
            })
        });

        let win = window();

        window_event_listener(leptos::ev::resize, move |_| {
            screen_rsc.update(|s| {
                if let Some(s) = s {
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
                        
                }
            })
        });
    }

    Memo::new(move |_| screen_rsc.get().unwrap_or_default())
}
