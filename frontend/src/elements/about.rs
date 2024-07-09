use leptos::{html::Dialog, *};

pub const TALLYWEB_VERSION: &str = env!("TALLYWEB_VERSION");

#[component]
pub fn AboutDialog(
    open: RwSignal<bool>,
    #[prop(optional)] accent_color: Option<Signal<String>>,
) -> impl IntoView {
    let about_node = create_node_ref::<Dialog>();
    create_effect(move |_| {
        if let Some(a) = about_node() {
            if open() {
                let _ = a.show_modal();
            } else {
                a.close()
            }
        };
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

    view! {
        <dialog node_ref=about_node style=border_style id="about">
            <div class="content">
                <span class="title">Author</span>
                <span class="info">P3rtang</span>
                <span class="title">Github</span>
                <a class="button" href="https://github.com/P3rtang/tallyWeb">
                    <i class="fa-solid fa-link"></i>
                </a>
                <span class="title">Version</span>
                <span class="info">{TALLYWEB_VERSION}</span>
            </div>
            <div class="actionbuttons">
                <button
                    style=button_style
                    on:click=move |_| {
                        if let Some(a) = about_node() {
                            a.close()
                        }
                    }
                >

                    Close
                </button>
            </div>
        </dialog>
    }
}