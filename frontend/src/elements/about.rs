use leptos::{html::Dialog, *};

stylance::import_style!(
    #[allow(dead_code)]
    about,
    "about.module.scss"
);
stylance::import_style!(
    #[allow(dead_code)]
    main,
    "../../style/_main.module.scss"
);

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
        <dialog
            class=stylance::classes!(about::big, main::overlay)
            node_ref=about_node
            style=border_style
            id="about"
        >
            <div class=about::content>
                <span class=about::title>Author</span>
                <span class=about::info>P3rtang</span>
                <span class=about::title>Github</span>
                <a
                    class=stylance::classes!(about::button, about::info)
                    href="https://github.com/P3rtang/tallyWeb"
                >
                    <i class="fa-solid fa-link"></i>
                </a>
                <span class=about::title>Version</span>
                <span class=about::info>{TALLYWEB_VERSION}</span>
            </div>
            <action-buttons>
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
            </action-buttons>
        </dialog>
    }
}
