#![allow(unused_braces)]

use leptos::*;
use leptos_router::ActionForm;
use web_sys::{Event, SubmitEvent};

use crate::app::{Preferences, SavePreferences, SessionUser};

#[component]
pub fn PreferencesWindow(cx: Scope) -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>(cx);
    let user = expect_context::<RwSignal<Option<SessionUser>>>(cx);

    let (accent_color, set_accent_color) = create_slice(
        cx,
        preferences,
        |pref| pref.accent_color.0.clone(),
        |pref, new| pref.accent_color.0 = new,
    );

    let action = create_server_action::<SavePreferences>(cx);
    let on_submit = |_: SubmitEvent| {};

    let border_style = move || format!("border: 2px solid {};", accent_color.get());

    let on_change = move |ev: Event| {
        let color = event_target_value(&ev);
        set_accent_color(color)
    };

    let (use_default, set_default) = create_slice(
        cx,
        preferences,
        |pref| pref.use_default_accent_color,
        |pref, new| pref.use_default_accent_color = new,
    );

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

    let style = create_read_slice(cx, preferences, move |pref| {
        if use_default() {
            format!("background-color: {}", pref.accent_color.0)
        } else {
            String::new()
        }
    });

    view! { cx,
        <ActionForm action=action on:submit=on_submit class="parent-form">
            <div class="preferences-form" style=border_style>
                <div class="preferences-row">
                    <label for="use_default_accent_color">Use Default Accent Color</label>
                    <label class="switch">
                        { move || { if use_default() { view! { cx,
                            <input type="checkbox" checked name="use_default_accent_color" on:input=on_toggle required />
                        }} else { view! { cx,
                            <input type="checkbox" name="use_default_accent_color" on:input=on_toggle required />
                        }}}}
                        <span class="slider" style=style/>
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
            </div>
        </ActionForm>
    }
}
