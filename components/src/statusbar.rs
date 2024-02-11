use std::{fmt::Display, path::PathBuf};

use leptos::*;

#[derive(Debug)]
pub struct KeyNotFound {
    key: String,
}

impl std::error::Error for KeyNotFound {}
impl Display for KeyNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "key: {} not found in statusbar", self.key)
    }
}

#[component]
pub fn StatusBar(status_bar: StatusBar) -> impl IntoView {
    view! {
        <Show when=move || !status_bar.is_empty()>
        <For
            each=status_bar.items
            key=|i| i.key.clone()
            children=move |i| i.view
        />
        // <div id="status-bar">
        //     <Show when=|| true>
        //         <svg style:width="24px" style:height="24px" viewBox="0 0 48 48" xmlns="http://www.w3.org/2000/svg">
        //           <title>wifi-disable</title>
        //           <g id="Layer_2" data-name="Layer 2">
        //             <g id="invisible_box" data-name="invisible box">
        //               <rect width="48" height="48" fill="none"/>
        //             </g>
        //             <g id="Q3_icons" data-name="Q3 icons">
        //               <g>
        //                 <path fill="currentColor" d="M40.7,26.5a2,2,0,0,0-.2-2.6A23,23,0,0,0,24.3,17L29,21.7a18.6,18.6,0,0,1,8.7,5A1.9,1.9,0,0,0,40.7,26.5Z"/>
        //                 <path fill="currentColor" d="M45.4,17.4A31.2,31.2,0,0,0,17.1,9.8l3.4,3.4L24,13a27.4,27.4,0,0,1,18.6,7.3,2,2,0,0,0,3-.2h0A2.1,2.1,0,0,0,45.4,17.4Z"/>
        //                 <circle fill="currentColor" cx="24" cy="38" r="5"/>
        //                 <path fill="currentColor" d="M5.4,3.6a1.9,1.9,0,0,0-2.8,0,1.9,1.9,0,0,0,0,2.8L9,12.8H8.7L6.8,14.1l-.3.3L4.7,15.7l-.3.2L2.6,17.4a2.1,2.1,0,0,0-.2,2.7h0a2,2,0,0,0,3,.2l1.7-1.4.4-.3,1.8-1.3.3-.2,2-1.1h0l.4-.2,3,3-.5.2-.6.3-.8.4-.6.3-.8.5-.5.4-.8.5-.5.4-.9.7-.4.3a11.4,11.4,0,0,1-1.1,1.1,2,2,0,0,0-.6,1.4,2.8,2.8,0,0,0,.4,1.2,1.9,1.9,0,0,0,3,.2l1.2-1,.3-.3.9-.7.4-.3,1.1-.7h.2l1.4-.8h.4l1.1-.5.5-.2h.3l3.3,3.3a16,16,0,0,0-9.1,5.3,1.9,1.9,0,0,0-.4,1.2,2,2,0,0,0,.4,1.3,2,2,0,0,0,3.1,0A11.5,11.5,0,0,1,24,29h1.2L38.6,42.4a1.9,1.9,0,0,0,2.8,0,1.9,1.9,0,0,0,0-2.8Z"/>
        //               </g>
        //             </g>
        //           </g>
        //         </svg>
        //     </Show>
        // </div>
        </Show>
    }
}

pub trait IntoStatusbar: IntoView {
    fn icon(&self) -> PathBuf;
}

#[derive(Debug, Clone)]
pub struct StatusBarItem {
    key: String,
    icon: View,
    view: View,

    is_shown: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct StatusBar {
    items: RwSignal<Vec<StatusBarItem>>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            items: Default::default(),
        }
    }

    fn is_empty(&self) -> bool {
        self.items.get_untracked().is_empty()
    }

    fn attach(&self, key: &str, icon: impl IntoView, overlay: impl IntoView) {
        let item = StatusBarItem {
            key: key.to_string(),
            icon: icon.into_view(),
            view: overlay.into_view(),
            is_shown: true,
        };
        self.items
            .update(|i| i.push(item))
    }

    fn hide(&self, key: &str) -> Result<(), KeyNotFound> {
        let (idx, _) = self
            .items
            .get_untracked()
            .iter()
            .enumerate()
            .find(|(_, item)| item.key == key)
            .ok_or(KeyNotFound { key: key.into() })?;

        self.items
            .update(|l| l.get_mut(idx).unwrap().is_shown = false);

        return Ok(())
    }
}
