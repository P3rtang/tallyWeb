#![allow(unused_braces)]

use std::time::Duration;

use leptos::*;
use leptos_router::ActionForm;
use web_sys::{Event, SubmitEvent};

use crate::app::{Preferences, SavePreferences, SessionUser};

#[component]
pub fn PreferencesWindow(cx: Scope) -> impl IntoView {
    let pref_resource = expect_context::<Resource<Option<SessionUser>, Preferences>>(cx);
    let preferences = expect_context::<RwSignal<Preferences>>(cx);

    let user = expect_context::<RwSignal<Option<SessionUser>>>(cx);
    create_effect(cx, move |_| {
        request_animation_frame(move || {
            user.set(SessionUser::from_storage(cx));
        })
    });

    let (use_default, set_default) = create_slice(
        cx,
        preferences,
        |pref| pref.use_default_accent_color,
        |pref, new| pref.use_default_accent_color = new,
    );

    let (accent_color, set_accent_color) = create_slice(
        cx,
        preferences,
        |pref| pref.accent_color.0.clone(),
        |pref, new| pref.accent_color.0 = new,
    );

    let action = create_server_action::<SavePreferences>(cx);

    let message = create_rw_signal(cx, None::<&'static str>);
    create_effect(cx, move |_| {
        if let Some(_) = action.value().get().map(|v| v.ok()).flatten() {
            message.set(Some("Settings Saved"));
            set_timeout(
                move || {
                    message.try_set(None);
                },
                Duration::from_secs(3),
            )
        } else if action.value().get().map(|v| v.is_err()).unwrap_or_default() {
            message.set(Some("Could not save Settings"))
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        match accent_color().split('#').collect::<Vec<_>>()[..] {
            ["", num]
                if (num.len() == 3 || num.len() == 6) && i32::from_str_radix(num, 16).is_ok() =>
            {
                return
            }
            [_] => message.set(Some("Color should start with `#`")),
            ["", _] => message.set(Some("Color should be 3 or 6 characters in hexadecimal")),
            [..] => message.set(Some("Invalid Color")),
        }
        ev.prevent_default();
        set_timeout(
            move || {
                message.try_set(None);
            },
            Duration::from_secs(5),
        )
    };

    let border_style = move || format!("border: 2px solid {};", accent_color.get());

    let on_change = move |ev: Event| {
        let color = event_target_value(&ev);
        if color.len() == 0 {
            ev.prevent_default()
        }
        set_accent_color(color)
    };

    let on_toggle = move |_| {
        if use_default() {
            set_default(false)
        } else {
            set_default(true);
            preferences.update(|p| {
                p.accent_color
                    .set_user(&user.get_untracked().unwrap_or_default())
            })
        }
    };

    let slider_style = create_read_slice(cx, preferences, move |pref| {
        if use_default() {
            format!("background-color: {}", pref.accent_color.0)
        } else {
            String::new()
        }
    });

    let confirm_style = create_read_slice(cx, preferences, move |pref| {
        format!("background-color: {}", pref.accent_color.0)
    });

    let undo_changes = move |_| {
        pref_resource.refetch();
    };

    view! { cx,
    <Show
        when=move || {
            user().is_some() && pref_resource.read(cx).is_some()
        }
        fallback=move |_| { view! { cx,  } }
    >
        <ActionForm action=action on:submit=on_submit class="parent-form">
            <input style="display:none" type="text" value={move || { user().map(|u| u.username).unwrap_or_default() }} name="username"/>
            <input style="display:none" type="text" value={move || { user().map(|u| u.token).unwrap_or_default() }} name="token"/>
            <div class="preferences-form" style=border_style>
                <div class="preferences-row">
                    <label for="use_default_accent_color">Use Default Accent Color</label>
                    <label class="switch">
                        { move || { if use_default() { view! { cx,
                            <input
                                type="checkbox"
                                checked="true"
                                value="true"
                                name="use_default_accent_color"
                                on:input=on_toggle/>
                        }} else { view! { cx,
                            <input
                                type="checkbox"
                                value="false"
                                name="use_default_accent_color"
                                on:input=on_toggle/>
                        }}}}
                        <span class="slider" style=slider_style/>
                    </label>
                </div>
                <div class="preferences-row">
                    <label for="accent_color">Accent Color</label>
                    { move || { if use_default() { view! { cx,
                        <input
                            type="text"
                            placeholder="e.g #AAA"
                            value={accent_color.get()}
                            name="accent_color"
                            spellcheck="false"
                            on:input=on_change
                            required
                            disabled
                        />
                     }} else { view! { cx,
                        <input
                            type="text"
                            placeholder="e.g #AAA"
                            value={accent_color.get_untracked()}
                            name="accent_color"
                            spellcheck="false"
                            on:input=on_change
                            required
                        />
                     }}}}
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
        </Show>
        <Show
            when=move || { message().is_some() }
            fallback=|_| ()
        >
            <b class="notification-box" style=border_style>{ move || { message().unwrap() } }</b>
        </Show>
    }
}
