use leptos::{html::Dialog, *};

use super::*;

#[component]
pub fn AboutDialog<F>(
    open: RwSignal<bool>,
    layout: F,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView
where
    F: Fn() -> ScreenLayout + Copy + 'static,
{
    let about_node = create_node_ref::<Dialog>();
    create_effect(move |_| {
        about_node().map(|a| {
            if open() {
                let _ = a.show_modal();
            } else {
                let _ = a.close();
            }
        });
    });

    let border_style = move || {
        accent_color
            .map(|ac| format!("border: 2px solid {};", ac()))
            .unwrap_or_default()
    };

    let button_style = move || {
        accent_color
            .map(|ac| format!("background: {};", ac()))
            .unwrap_or_default()
    };

    let class = move || match layout() {
        ScreenLayout::Small => format!("overlay"),
        ScreenLayout::Big => format!("overlay big"),
    };

    view! {
        <dialog node_ref=about_node style=border_style class=class id="about">
            <div class="content">
                <label class="title">Author</label>
                <label class="info">P3rtang</label>
                <label class="title">Github</label>
                <a class="button" href="https://github.com/P3rtang/tallyWeb">
                    <i class="fa-solid fa-link"></i>
                </a>
                <label class="title">Version</label>
                <label class="info">0.2.4</label>
            </div>
            <div class="actionbuttons">
                <button
                    style=button_style
                    on:click=move |_| { about_node().map(|a| a.close()); }
                >
                    Close
                </button>
            </div>
        </dialog>
    }
}
