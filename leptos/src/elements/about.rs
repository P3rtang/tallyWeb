use leptos::{html::Dialog, *};

use crate::elements::ScreenLayout;

#[component]
pub fn AboutDialog<F>(open: RwSignal<bool>, layout: F) -> impl IntoView
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

    let preferences = expect_context::<RwSignal<crate::app::Preferences>>();
    let border_style = create_read_slice(preferences, |pref| {
        format!("border: 2px solid {};", pref.accent_color.0)
    });

    let button_style = create_read_slice(preferences, |pref| {
        format!("background: {};", pref.accent_color.0)
    });

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
                <label class="info">0.2.3</label>
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
