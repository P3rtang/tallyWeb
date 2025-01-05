#![allow(dead_code)]

use super::{connect_on_window_resize, AppError};
use components::{MessageJar, SidebarLayout};
use leptos::{logging, prelude::*};
use wasm_bindgen::JsCast;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ScreenStyle {
    Portrait,
    Small,
    Big,
}

impl From<ScreenStyle> for SidebarLayout {
    fn from(val: ScreenStyle) -> Self {
        match val {
            ScreenStyle::Portrait => SidebarLayout::Portrait,
            ScreenStyle::Small => SidebarLayout::Hover,
            ScreenStyle::Big => SidebarLayout::Landscape,
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Screen {
    pub style: RwSignal<ScreenStyle>,
    pub size: RwSignal<(usize, usize)>,
}

impl Screen {
    #[cfg(feature = "ssr")]
    pub fn new(size: (usize, usize)) -> Result<Self, AppError> {
        let style = match size {
            (w, h) if w < 600 && h > w => ScreenStyle::Portrait,
            (w, _) if w < 1200 => ScreenStyle::Small,
            _ => ScreenStyle::Big,
        };

        Ok(Self {
            style: RwSignal::new(style),
            size: RwSignal::new(size),
        })
    }

    pub fn update(&self) -> Result<(), AppError> {
        let width = leptos::leptos_dom::helpers::window()
            .inner_width()
            .map_err(|val| AppError::WindowSize(val.as_string().unwrap_or_default()))?
            .as_f64()
            .ok_or(AppError::WindowSize(
                "Unable to convert JsValue to f64".to_string(),
            ))? as usize;

        let height = leptos::leptos_dom::helpers::window()
            .inner_height()
            .map_err(|val| AppError::WindowSize(val.as_string().unwrap_or_default()))?
            .as_f64()
            .ok_or(AppError::WindowSize(
                "Unable to convert JsValue to f64".to_string(),
            ))? as usize;

        let style = match (width, height) {
            _ if width < 600 && height > width => ScreenStyle::Portrait,
            _ if width < 1200 => ScreenStyle::Small,
            _ => ScreenStyle::Big,
        };

        self.style.set(style);
        self.size.set((width, height));

        let document = document();
        let document: &web_sys::HtmlDocument = document.unchecked_ref();

        let size_json = serde_json::to_string(&(width, height)).unwrap();
        let size_cookie = cookie::Cookie::build(("screen_size", &size_json))
            .path("/")
            .same_site(cookie::SameSite::Strict)
            .build();
        let size_cookie_encoded = cookie::Cookie::encoded(&size_cookie);

        let mut cookie_str = size_cookie_encoded.to_string();
        let age_str = format!("; Max-Age={}", 30 * 24 * 60 * 60);
        cookie_str.push_str(&age_str);

        document.set_cookie(&cookie_str).ok();

        Ok(())
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            style: RwSignal::new(ScreenStyle::Big),
            size: RwSignal::new((1920, 1080)),
        }
    }
}

#[server]
pub async fn get_screen_cookie() -> Result<Screen, ServerFnError> {
    use leptos_actix::extract;

    let header = extract::<actix_web::HttpRequest>().await?;
    let cookie = match header.cookie("screen_size") {
        Some(s) => s.value().to_string(),
        None => return Ok(Screen::default()),
    };

    let size: (usize, usize) = serde_json::from_str(&cookie)?;

    return Ok(Screen::new(size)?);
}

pub async fn provide_screen() -> Result<(), AppError> {
    let owner = Owner::current().unwrap();

    let screen = get_screen_cookie().await.unwrap_or_default();

    #[cfg(feature = "csr")]
    Effect::new(move |_| {
        let _ = screen.update();
        connect_on_window_resize(Box::new(move || {
            if let Err(err) = screen.update() {
                if let Some(msg) = use_context::<MessageJar>() {
                    msg.set_err(err.clone())
                }
                logging::warn!("{}", err)
            }
        }))
    });

    owner.with(move || provide_context(screen));

    Ok(())
}
