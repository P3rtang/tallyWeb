#![allow(dead_code)]

use leptos::prelude::*;

#[derive(Clone, Default)]
pub struct PageContext {
    pub overlay: Overlay,
}

impl PageContext {
    pub fn new() -> Self {
        Self::default()
    }
}

impl IntoRender for PageContext {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let body = {
            let this = self.clone();
            move || this.clone().overlay.body.get().run()
        };

        let overlay = self.overlay.clone();
        let is_open = move || overlay.is_open.get();

        view! {
            <Show when=is_open >
                <overlay-element>{body.clone()}</overlay-element>
            </Show>
        }
        .into_any()
    }
}

#[derive(Clone)]
pub struct Overlay {
    pub is_open: leptos::prelude::RwSignal<bool>,
    pub body: ArcRwSignal<ViewFn>,
}

impl Default for Overlay {
    fn default() -> Self {
        Self {
            is_open: RwSignal::new(false),
            body: ArcRwSignal::new(ViewFn::from(move || View::new(()))),
        }
    }
}
