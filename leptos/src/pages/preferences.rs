#![allow(unused_braces)]

use std::time::Duration;

use leptos::{html::Input, *};
use leptos_router::ActionForm;
use web_sys::{Event, SubmitEvent};

use crate::{
    app::{save_preferences, Preferences, SavePreferences, SessionUser},
    elements::ScreenLayout,
};

#[component]
pub fn PreferencesWindow<F>(cx: Scope, layout: F) -> impl IntoView
where
    F: Fn() -> ScreenLayout + Copy + 'static,
{
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

    let (show_sep, set_sep) = create_slice(
        cx,
        preferences,
        |pref| pref.show_separator,
        |pref, new| pref.show_separator = new,
    );

    let action = create_server_action::<SavePreferences>(cx);
    let message = create_rw_signal(cx, None::<&'static str>);

    let i_use_default: NodeRef<Input> = create_node_ref(cx);
    let i_accent_color: NodeRef<Input> = create_node_ref(cx);
    let i_show_separator: NodeRef<Input> = create_node_ref(cx);

    create_effect(cx, move |_| {
        i_accent_color().map(|i| i.set_value(&accent_color()));
        i_show_separator().map(|i| i.set_checked(show_sep()))
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        match accent_color().split('#').collect::<Vec<_>>()[..] {
            ["", num]
                if (num.len() == 3 || num.len() == 6) && i32::from_str_radix(num, 16).is_ok() => {}
            [_] => message.set(Some("Color should start with `#`")),
            ["", _] => message.set(Some("Color should be 3 or 6 characters in hexadecimal")),
            [..] => message.set(Some("Invalid Color")),
        }
        set_timeout(
            move || {
                message.try_set(None);
            },
            Duration::from_secs(5),
        );

        set_default(i_use_default().map(|i| i.checked()).unwrap_or_default());
        set_accent_color(i_accent_color().map(|i| i.value()).unwrap_or_default());
        set_sep(i_show_separator().map(|i| i.checked()).unwrap_or_default());

        let action = create_action(cx, async move |_: &()| -> Result<(), ServerFnError> {
            let user = user
                .get_untracked()
                .ok_or(ServerFnError::MissingArg(String::from(
                    "User not available",
                )))?;

            save_preferences(user.username, user.token, preferences.get_untracked()).await?;
            message.set(Some("Settings Saved"));
            set_timeout(
                move || {
                    message.try_set(None);
                },
                Duration::from_secs(3),
            );
            return Ok(());
        });
        action.dispatch(());
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
            set_default(false);
        } else {
            set_default(true);
            preferences.update(|p| {
                p.accent_color
                    .set_user(&user.get_untracked().unwrap_or_default())
            });
            i_accent_color().map(|i| i.set_value(&accent_color()));
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
            <input style="display:none" type="text" value={move || { user().map(|u| u.username).unwrap_or_default() }} name="username" autocomplete="none"/>
            <input style="display:none" type="text" value={move || { user().map(|u| u.token).unwrap_or_default() }} name="token"/>
            <div class={ move || String::from("editing-form ") + layout().get_class() } style=border_style>
                <div class="content">
                    <label for="use_default_accent_color" class="title">Use Default Accent Color</label>
                    <label class="switch">
                        <input
                            type="checkbox"
                            prop:checked=use_default
                            name="use_default_accent_color"
                            id="use_default_accent_color"
                            class="edit"
                            node_ref=i_use_default
                            on:input=on_toggle/>
                        <span class="slider" style=slider_style/>
                    </label>
                    <label for="accent_color" class="title">Accent Color</label>
                    <input
                        type="color"
                        // name="accent_color"
                        id="accent_color"
                        class="edit"
                        node_ref=i_accent_color
                        on:input=on_change
                        prop:disabled=use_default
                    />
                    <label for="show_separator" class="title">Show Treeview Separator</label>
                    <input
                        type="checkbox"
                        name="show_separator"
                        id="show_separator"
                        class="edit"
                        node_ref=i_show_separator
                    />
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
