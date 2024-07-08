use super::{connect_on_window_resize, AppError};
use components::MessageJar;
use leptos::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ScreenStyle {
    Portrait,
    Small,
    Big,
}

impl ScreenStyle {
    pub fn to_sidebar(self) -> components::SidebarStyle {
        match self {
            ScreenStyle::Portrait => components::SidebarStyle::Portrait,
            ScreenStyle::Small => components::SidebarStyle::Hover,
            ScreenStyle::Big => components::SidebarStyle::Landscape,
        }
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Screen {
    pub style: RwSignal<ScreenStyle>,
    pub size: RwSignal<(usize, usize)>,
}

impl Screen {
    pub fn new(size: (usize, usize)) -> Result<Self, AppError> {
        let style = match size {
            (w, h) if w < 600 && h > 800 => ScreenStyle::Portrait,
            (w, _) if w < 1200 => ScreenStyle::Small,
            _ => ScreenStyle::Big,
        };

        Ok(Self {
            style: create_rw_signal(style),
            size: create_rw_signal(size),
        })
    }

    pub fn update(&self) -> Result<(), AppError> {
        let width = leptos_dom::window()
            .inner_width()
            .map_err(|val| AppError::WindowSize(val.as_string().unwrap_or_default()))?
            .as_f64()
            .ok_or(AppError::WindowSize(
                "Unable to convert JsValue to f64".to_string(),
            ))? as usize;

        let height = leptos_dom::window()
            .inner_height()
            .map_err(|val| AppError::WindowSize(val.as_string().unwrap_or_default()))?
            .as_f64()
            .ok_or(AppError::WindowSize(
                "Unable to convert JsValue to f64".to_string(),
            ))? as usize;

        let style = match (width, height) {
            _ if width < 600 && height > 800 => ScreenStyle::Portrait,
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
            style: create_rw_signal(ScreenStyle::Big),
            size: create_rw_signal((1920, 1080)),
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

async fn get_screen() -> Screen {
    get_screen_cookie().await.unwrap_or_default()
}

#[component(transparent)]
pub fn ProvideScreenSignal(children: ChildrenFn) -> impl IntoView {
    view! {
        <Await future=get_screen let:screen>

            {
                let s = *screen;
                create_effect(move |_| {
                    let _ = s.update();
                    connect_on_window_resize(
                        Box::new(move || {
                            if let Err(err) = s.update() {
                                if let Some(msg) = use_context::<MessageJar>() {
                                    msg.set_err(err.clone())
                                }
                                logging::warn!("{}", err)
                            }
                        }),
                    )
                });
                provide_context(s);
                children()
            }

        </Await>
    }
}
