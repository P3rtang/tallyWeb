use super::*;

use leptos::prelude::*;
use leptos_router::hooks::use_params;

stylance::import_style!(style, "./infobox.module.scss");

#[component]
pub fn InfoHeader(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();

    let name = create_read_slice(store, move |s| s.name(&key.get()));

    let params = use_params::<UserName>();
    let user_name = move || params.get().map(|p| p.id).ok();

    let edit_link = move || {
        if let Some(user_name) = user_name() {
            format!("{}/edit?slct={}", user_name, key().0)
        } else {
            format!("edit?slct={}", key().0)
        }
    };

    let on_click = move |_| {
        use_referer(RefererOptions { is_refering: true });
    };

    view! {
        <div class=style::header>
            <div>
                <span>
                    {name}
                </span>
                <button class="hover-darken">
                    <div>
                        <a href=edit_link on:click=on_click>
                            <img width="32px" height="32px" src="/icons/white-edit-svgrepo-com.svg" />
                        </a>
                    </div>
                </button>
            </div>
        </div>
    }
}
