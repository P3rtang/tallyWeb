#![allow(unused_braces)]
use components::{MessageJar, SavingMessage, SidebarStyle, Slider};
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
    let message = expect_context::<MessageJar>();
    let user = expect_context::<RwSignal<UserSession>>();
    let pref_resource = expect_context::<PrefResource>();

    let action = create_server_action::<api::SavePreferences>();

    let (accent_color, set_accent_color) = create_slice(
        preferences,
        |pref| pref.accent_color.0.clone(),
        |pref, new| pref.accent_color.0 = new,
    );

    let on_change = move |ev: Event| {
        let color = event_target_value(&ev);
        if color.is_empty() {
            ev.prevent_default()
        }
        set_accent_color(color)
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        action.dispatch(api::SavePreferences {
            session: user.get_untracked(),
            preferences: preferences.get_untracked(),
        });

        message
            .without_timeout()
            .as_modal()
            .set_msg_view(SavingMessage);

        let msg_key = message.get_last_key();

        create_effect(move |_| match action.value().get() {
            Some(Ok(_)) => {
                message.fade_out(msg_key.get());
                action.value().set_untracked(None)
            }
            Some(Err(err)) => {
                message.fade_out(msg_key.get());
                message.set_err(AppError::from(err));
                action.value().set_untracked(None)
            }
            None => {}
        });
    };

    let on_default_checked = move |_: Event| {
        preferences.update(|p| p.use_default_accent_color = !p.use_default_accent_color);
        preferences.update(|p| p.accent_color.set_user(&user.get_untracked()))
    };

    let on_separator_checked =
        move |_: Event| preferences.update(|p| p.show_separator = !p.show_separator);

    let on_multi_checked = move |_: Event| preferences.update(|p| p.multi_select = !p.multi_select);

    let undo_changes = move |_| pref_resource.refetch();

    let border_style = move || format!("border: 2px solid {};", accent_color.get());
    let confirm_style = move || format!("background-color: {}", accent_color.get());

    view! {
        <elements::Navbar has_sidebar=false></elements::Navbar>
        <ActionForm action=action on:submit=on_submit class="parent-form">
            <div
                class=move || String::from("editing-form ") + layout().get_widget_class()
                style=border_style
            >
                <div class="content">
                    <label for="use_default_accent_color" class="title">
                        Use Default Accent Color
                    </label>
                    <Slider
                        value=preferences.get_untracked().use_default_accent_color
                        name="use_default_accent_color"
                        on_checked=on_default_checked
                        accent_color
                    />
                    <label for="accent_color" class="title">
                        Accent Color
                    </label>
                    <input
                        type="color"
                        name="accent_color"
                        id="accent_color"
                        class="edit"
                        on:input=on_change
                        disabled=move || preferences().use_default_accent_color
                        value=accent_color
                        prop:value=accent_color
                    />

                    <label for="show_separator" class="title">
                        Show Treeview Separator
                    </label>
                    <Slider
                        value=preferences.get_untracked().show_separator
                        on_checked=on_separator_checked
                        accent_color
                    />

                    <label class="title">Use Multi Select (experimental)</label>
                    <Slider
                        value=preferences.get_untracked().multi_select
                        on_checked=on_multi_checked
                        accent_color
                    />

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
