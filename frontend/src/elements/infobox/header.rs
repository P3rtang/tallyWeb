use super::*;

#[component]
pub fn InfoHeader(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let history = use_history();
    let session = expect_context::<RwSignal<UserSession>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let selection = expect_context::<Memo<Selection>>();
    let name = Signal::derive(move || store.get().name(&key.get()));

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
        history.save_location();
    };

    view! {
        <div class=style::header>
            <div>
                <span>{name}</span>
                <div class=style::actions>
                    <button class="hover-darken icon">
                        <a href=edit_link on:click=on_click>
                            <Icon kind=IconKind::Edit />
                        </a>
                    </button>
                </div>
            </div>
        </div>
    }
}
