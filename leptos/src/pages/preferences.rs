#![allow(unused_braces)]
use components::{Message, SavingMessage, SidebarStyle, Slider};
use leptos::*;
use leptos_router::{ActionForm, A};
use web_sys::{Event, SubmitEvent};

use super::*;

#[component]
pub fn PreferencesWindow<F>(layout: F) -> impl IntoView
where
    F: Fn() -> SidebarStyle + Copy + 'static,
{
    let preferences = expect_context::<RwSignal<Preferences>>();
    let message = expect_context::<Message>();
    let user = expect_context::<RwSignal<UserSession>>();

    let action = create_server_action::<api::SavePreferences>();

    let use_default_accent_color = create_slice(
        preferences,
        |pref| pref.use_default_accent_color,
        |pref, new| pref.use_default_accent_color = new,
    );

    let (accent_color, set_accent_color) = create_slice(
        preferences,
        |pref| pref.accent_color.0.clone(),
        |pref, new| pref.accent_color.0 = new,
    );

    let separator = create_slice(
        preferences,
        |pref| pref.show_separator,
        |pref, new| pref.show_separator = new,
    );

    let multi_select = create_slice(
        preferences,
        |pref| pref.multi_select,
        |pref, new| pref.multi_select = new,
    );

    let undo_changes = move |_| {
        // pref_resource.refetch();
    };

    let on_change = move |ev: Event| {
        let color = event_target_value(&ev);
        if color.is_empty() {
            ev.prevent_default()
        }
        set_accent_color(color)
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let action = create_action(move |_| {
            let user = user.get_untracked();

            async move {
                match api::save_preferences(user, preferences.get_untracked()).await {
                    Ok(_) => message.set_msg("Settings Saved"),
                    Err(err) => message.set_server_err(&err.to_string()),
                };

                Ok::<(), ServerFnError>(())
            }
        });

        action.dispatch(());
        message
            .without_timeout()
            .as_modal()
            .set_msg_view(SavingMessage)
    };

    let border_style = move || format!("border: 2px solid {};", accent_color.get());
    let confirm_style = move || format!("background-color: {}", accent_color.get());

    view! {
        <elements::Navbar></elements::Navbar>
        <ActionForm action=action on:submit=on_submit class="parent-form">
            <div
                class=move || String::from("editing-form ") + layout().get_widget_class()
                style=border_style
            >
                <div class="content">
                    <label for="use_default_accent_color" class="title">
                        Use Default Accent Color
                    </label>
                    <Slider checked=use_default_accent_color accent_color/>
                    {move || {
                        use_default_accent_color
                            .0
                            .with(|b| {
                                if *b {
                                    preferences
                                        .update(|p| {
                                            p.accent_color.set_user(&user.get_untracked())
                                        });
                                }
                            })
                    }}

                    <label for="accent_color" class="title">
                        Accent Color
                    </label>
                    <input
                        type="color"
                        name="accent_color"
                        id="accent_color"
                        class="edit"
                        on:input=on_change
                        prop:disabled=move || use_default_accent_color.0()
                        prop:value=accent_color
                    />

                    <label for="show_separator" class="title">
                        Show Treeview Separator
                    </label>
                    <Slider checked=separator accent_color/>

                    <label class="title">Use Multi Select (experimental)</label>
                    <Slider checked=multi_select accent_color/>

                    <label class="title">Change Username</label>
                    <A class="edit" href="/change-username">
                        <i class="fa-solid fa-arrow-right"></i>
                    </A>
                    <label class="title">Change Password</label>
                    <A class="edit" href="/change-password">
                        <i class="fa-solid fa-arrow-right"></i>
                    </A>
                </div>
                <div class="action-buttons">
                    <button type="button" on:click=undo_changes>
                        <span>Undo</span>
                    </button>
                    <button type="submit" style=confirm_style>
                        <span>Save</span>
                    </button>
                </div>
            </div>
        </ActionForm>
    }
}
