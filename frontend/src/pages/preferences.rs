#![allow(unused_braces)]
use components::{MessageJar, SavingMessage, Slider};
use leptos::*;
use leptos_router::{ActionForm, A};
use web_sys::{Event, SubmitEvent};

stylance::import_style!(
    #[allow(dead_code)]
    style,
    "../../style/edit.module.scss"
);

use super::*;

#[component]
pub fn PreferencesWindow() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();
    let message = expect_context::<MessageJar>();
    let session = expect_context::<RwSignal<UserSession>>();
    let pref_resource = expect_context::<PrefResource>();
    let screen = expect_context::<Screen>();

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
            session_user_uuid: session.get_untracked().user_uuid,
            session_username: session.get_untracked().username,
            session_token: session.get_untracked().token,
            preferences: preferences.get_untracked(),
        });

        let msg_key = message
            .with_handle()
            .without_timeout()
            .set_msg_view(SavingMessage);

        create_effect(move |_| match action.value().get() {
            Some(Ok(_)) => {
                message.fade_out(msg_key);
                action.value().set_untracked(None)
            }
            Some(Err(err)) => {
                message.fade_out(msg_key);
                message.set_err(AppError::from(err));
                action.value().set_untracked(None)
            }
            None => {}
        });
    };

    let on_default_checked = move |_: Event| {
        preferences.update(|p| p.use_default_accent_color = !p.use_default_accent_color);
        preferences.update(|p| p.accent_color.set_user(&session.get_untracked()))
    };

    let on_separator_checked =
        move |_: Event| preferences.update(|p| p.show_separator = !p.show_separator);

    let on_multi_checked = move |_: Event| preferences.update(|p| p.multi_select = !p.multi_select);

    let undo_changes = move |_| pref_resource.refetch();

    let form_style = move || {
        stylance::classes!(
            style::form,
            match (screen.style)() {
                ScreenStyle::Portrait => Some(style::portrait),
                ScreenStyle::Small => Some(style::small),
                ScreenStyle::Big => Some(style::big),
            }
        )
    };

    view! {
        <elements::Navbar has_sidebar=false></elements::Navbar>
        <h1 style:color="white" style:padding="12px 48px">
            Settings
        </h1>
        <div style:display="flex" style:height="100%" style:justify-content="center">
            <edit-form class=form_style>
                <ActionForm action=action on:submit=on_submit>
                    <SessionFormInput session />
                    <table class=style::content>
                        <tr class=style::row>
                            <td>
                                <label for="use-default-color" class="title">
                                    Use Default Accent Color
                                </label>
                            </td>
                            <td>
                                <Slider
                                    checked=preferences.get_untracked().use_default_accent_color
                                    attr:name="preferences[use_default_accent_color]"
                                    attr:id="use-default-color"
                                    on:change=on_default_checked
                                />
                            </td>
                        </tr>

                        <tr class=style::row>
                            <td>
                                <label for="accent-color" class="title">
                                    Accent Color
                                </label>
                            </td>
                            <td>
                                <input
                                    type="color"
                                    name="preferences[accent_color]"
                                    id="accent-color"
                                    class="edit"
                                    on:input=on_change
                                    disabled=move || preferences().use_default_accent_color
                                    value=accent_color
                                    prop:value=accent_color
                                />
                            </td>
                        </tr>

                        <SaveOnPause />

                        <tr>
                            <td colspan="2">
                                <hr />
                            </td>
                        </tr>

                        <tr class=style::row>
                            <td>
                                <label for="show-separator" class="title">
                                    Show Treeview Separator
                                </label>
                            </td>
                            <td>
                                <Slider
                                    checked=preferences.get_untracked().show_separator
                                    attr:name="preferences[show_separator]"
                                    attr:id="show-separator"
                                    on:change=on_separator_checked
                                />
                            </td>
                        </tr>

                        <tr class=style::row>
                            <td>
                                <label for="multi-select" class="title">
                                    Use Multi Select (experimental)
                                </label>
                            </td>
                            <td>
                                <Slider
                                    checked=preferences.get_untracked().multi_select
                                    attr:name="preferences[multi_select]"
                                    attr:id="multi-select"
                                    on:change=on_multi_checked
                                />
                            </td>
                        </tr>

                        <tr>
                            <td colspan="2">
                                <hr />
                            </td>
                        </tr>

                        <tr class=style::row>
                            <td>
                                <span for="change-username" class="title">
                                    Change Username
                                </span>
                            </td>
                            <td>
                                <A class=style::edit href="/change-username">
                                    <i class="fa-solid fa-arrow-right"></i>
                                </A>
                            </td>
                        </tr>

                        <tr class=style::row>
                            <td>
                                <span class="title">Change Password</span>
                            </td>
                            <td>
                                <A class=style::edit href="/change-password">
                                    <i class="fa-solid fa-arrow-right"></i>
                                </A>
                            </td>
                        </tr>
                    </table>

                    <action-buttons
                        style:display="flex"
                        style:justify-content="space-between"
                        class=move || {
                            stylance::classes!(
                                style::action_buttons, match (screen.style) () {
                                ScreenStyle::Portrait => Some(style::fixed), ScreenStyle::Small =>
                                None, ScreenStyle::Big => None, }
                            )
                        }
                    >

                        <action-start></action-start>
                        <action-end>
                            <button type="button" on:click=undo_changes>
                                <span>Undo</span>
                            </button>
                            <button type="submit" class=style::confirm>
                                <span>Save</span>
                            </button>
                        </action-end>
                    </action-buttons>
                </ActionForm>
            </edit-form>
        </div>
    }
}

#[component]
fn SaveOnPause() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();
    let (checked, set_checked) =
        create_slice(preferences, |p| p.save_on_pause, |p, c| p.save_on_pause = c);
    let on_change = move |_| set_checked(!checked());

    view! {
        <tr class=style::row>
            <td>
                <label for="save-on-pause">Save on pause</label>
            </td>
            <td>
                <Slider
                    checked
                    attr:name="preferences[save_on_pause]"
                    attr:id="save-on-pause"
                    on:change=on_change
                />
            </td>
        </tr>
    }
}
