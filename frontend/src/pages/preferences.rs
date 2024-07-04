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

    view! {
        <elements::Navbar has_sidebar=false></elements::Navbar>
        <ActionForm action=action on:submit=on_submit class="parent-form">
            <div
                class=move || String::from("editing-form ") + layout().get_widget_class()
                style=border_style
            >
                <div class="content">
                    <label for="use-default-color" class="title">
                        Use Default Accent Color
                    </label>
                    <Slider
                        checked=preferences.get_untracked().use_default_accent_color
                        attr:name="use-default-color"
                        attr:id="use-default-color"
                        on_checked=on_default_checked
                    />
                    <label for="accent-color" class="title">
                        Accent Color
                    </label>
                    <input
                        type="color"
                        name="accent-color"
                        id="accent-color"
                        class="edit"
                        on:input=on_change
                        disabled=move || preferences().use_default_accent_color
                        value=accent_color
                        prop:value=accent_color
                    />

                    <label for="show-separator" class="title">
                        Show Treeview Separator
                    </label>
                    <Slider
                        checked=preferences.get_untracked().show_separator
                        attr:name="show-separator"
                        attr:id="show-separator"
                        on_checked=on_separator_checked
                    />

                    <label for="multi-select" class="title">
                        Use Multi Select (experimental)
                    </label>
                    <Slider
                        checked=preferences.get_untracked().multi_select
                        attr:name="multi-select"
                        attr:id="multi-select"
                        on_checked=on_multi_checked
                    />

                    <span for="change-username" class="title">
                        Change Username
                    </span>
                    <A class="edit" href="/change-username">
                        <i class="fa-solid fa-arrow-right"></i>
                    </A>
                    <span class="title">Change Password</span>
                    <A class="edit" href="/change-password">
                        <i class="fa-solid fa-arrow-right"></i>
                    </A>
                </div>
                <div class="action-buttons">
                    <button type="button" on:click=undo_changes>
                        <span>Undo</span>
                    </button>
                // <button type="submit" style=confirm_style>
                // <span>Save</span>
                // </button>
                </div>
            </div>
        </ActionForm>
    }
}
