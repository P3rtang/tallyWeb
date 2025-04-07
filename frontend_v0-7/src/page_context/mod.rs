#![allow(dead_code)]

use super::*;
use elements::{button::*, icon::*};

#[derive(Clone, Default)]
pub struct PageContext {
    pub overlay: Overlay,
    pub sidebar: Sidebar,
    pub history: hooks::History,
}

impl PageContext {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IntoRender for PageContext {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let Overlay { is_open, body } = self.overlay;

        view! {
            <Show when=is_open >
                <overlay-element
                    style:position="relative"
                    style:z-index="100"
                >
                    {body.clone().get().run()}
                </overlay-element>
            </Show>
        }
        .into_any()
    }
}

#[derive(Clone)]
pub struct Overlay {
    pub is_open: RwSignal<bool>,
    pub body: ArcRwSignal<ViewFn>,
}

impl Overlay {
    pub fn set_body(&self, view: impl Into<ViewFn>) {
        self.body.set(view.into())
    }
}

impl Default for Overlay {
    fn default() -> Self {
        Self {
            is_open: RwSignal::new(false),
            body: ArcRwSignal::new(ViewFn::from(move || View::new(()))),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Referer(pub(crate) RwSignal<Option<String>>);

#[derive(Clone, Copy)]
pub struct Sidebar {
    width: RwSignal<usize>,
    is_shown: RwSignal<bool>,
}

impl Sidebar {
    pub fn width(&self) -> ReadSignal<usize> {
        self.width.read_only()
    }

    pub fn width_attr(self) -> Signal<String> {
        let screen = hooks::use_screen();

        Signal::derive(move || {
            if screen.get().viewport() <= ViewPort::Small && self.is_shown().get() {
                return "100%".to_string();
            }

            format!("{}px", self.width().get())
        })
    }

    pub fn set_width(&self) -> WriteSignal<usize> {
        self.width.write_only()
    }

    pub fn is_shown(&self) -> ReadSignal<bool> {
        self.is_shown.read_only()
    }

    pub fn toggle_button(&self) -> impl IntoView {
        let is_open = self.is_shown;
        let icon = Signal::derive(move || {
            if is_open.get() {
                IconKind::SidebarClosed
            } else {
                IconKind::SidebarOpen
            }
        });

        let on_toggle_sidebar = move |_| is_open.set(!is_open.get());

        view! {
            <Button
                size=ButtonSize::Small
                hover=ButtonHover::Lighten
                on:mousedown=on_toggle_sidebar
                attr:aria_label="toggle sidebar"
                style:background="transparent"
            >
                <Icon kind=icon />
            </Button>
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            width: RwSignal::new(400),
            is_shown: RwSignal::new(true),
        }
    }
}
