#![allow(unused_braces)]
use components::{MessageJar, SavingMessage, ShowSidebar, Sidebar, SidebarLayout, Slider};
use elements::{
    Color, FromClosure, Navbar, OnClose, OnResize, Page, PageContent, PageNavbar, PageSidebar,
};
use leptos::*;
use leptos_router::{ActionForm, Outlet, A};
use web_sys::{Event, SubmitEvent};

stylance::import_style!(
    #[allow(dead_code)]
    style,
    "./style/edit.module.scss"
);

stylance::import_style!(
    #[allow(dead_code)]
    list_style,
    "../../style/_main.module.scss"
);

use super::*;

#[component]
pub fn PreferencesWindow() -> impl IntoView {
    let message = expect_context::<MessageJar>();
    let preferences = expect_context::<RwSignal<Preferences>>();
    let session = expect_context::<RwSignal<UserSession>>();
    let pref_resource = expect_context::<PrefResource>();
    let screen = expect_context::<Screen>();

    let action = create_server_action::<api::SavePreferences>();

    let accent = create_read_slice(preferences, |p| {
        Color::try_from(p.accent_color.clone().0.as_str()).unwrap_or_default()
    });

    let hide_border = create_read_slice(preferences, |p| !p.show_body_border);

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

    let (show_sidebar, set_show_sidebar) =
        create_signal(screen.style.get_untracked() == ScreenStyle::Big);

    let sidebar_layout: Signal<SidebarLayout> = create_read_slice(screen.style, |s| (*s).into());

    let (width, set_width) = create_signal(400);

    let on_resize = OnResize::from_closure(set_width);
    let on_close_sidebar = StoredValue::new(OnClose::from_closure(set_show_sidebar));

    view! {
        <Page accent>
            <PageContent hide_border slot>
                <edit-form class=form_style>
                    <ActionForm action=action on:submit=on_submit>
                        <SessionFormInput session />
                        <div class=style::content style:flex-direction="column">
                            <div>
                                <div class=style::grid>
                                    <Outlet />
                                </div>
                            </div>
                            <action-buttons
                                style:display="flex"
                                style:justify-content="space-between"
                                style:width="100%"
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
                        </div>
                    </ActionForm>
                </edit-form>
            </PageContent>
            <PageSidebar is_shown=show_sidebar on_resize slot>
                <Sidebar
                    display=Signal::derive(move || ShowSidebar(show_sidebar()))
                    width
                    layout=sidebar_layout
                >
                    <div style:height="60px" />
                    <div class=style::list_view>
                        <For
                            each=|| vec!["styling", "account", "misc"]
                            key=|item| item.to_string()
                            children=move |item| view! { <ListViewRow name=item /> }
                        />
                    </div>
                </Sidebar>
            </PageSidebar>
            <PageNavbar slot>
                <Navbar show_sidebar on_close_sidebar=on_close_sidebar.get_value() />
            </PageNavbar>
        </Page>
    }
}

#[component]
pub fn StylingPreferences() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();
    let session = expect_context::<RwSignal<UserSession>>();

    let (accent_color, set_accent_color) = create_slice(
        preferences,
        |pref| pref.accent_color.0.clone(),
        |pref, new| pref.accent_color.0 = new,
    );

    let on_default_checked = move |_: Event| {
        preferences.update(|p| p.use_default_accent_color = !p.use_default_accent_color);
        preferences.update(|p| p.accent_color.set_user(&session.get_untracked()))
    };

    let on_change = move |ev: Event| {
        let color = event_target_value(&ev);
        if color.is_empty() {
            ev.prevent_default()
        }
        set_accent_color(color)
    };

    let handle_border_change = move |ev: Event| {
        preferences.update(|p| p.show_body_border = event_target_checked(&ev));
    };

    view! {
        <>
            <div>
                <input
                    type="hidden"
                    name="preference[save_on_pause]"
                    value=preferences.get_untracked().save_on_pause
                />
                <input
                    type="hidden"
                    name="preference[show_separator]"
                    value=preferences.get_untracked().show_separator
                />
                <input
                    type="hidden"
                    name="preference[multi_select]"
                    value=preferences.get_untracked().multi_select
                />
                <input
                    type="hidden"
                    name="preference[show_body_border]"
                    value=preferences.get_untracked().show_body_border
                />
            </div>

            <label for="use-default-color" class="title" style:grid-column=1>
                Use Default Accent Color
            </label>
            <div style:grid-column=2>
                <Slider
                    checked=preferences.get_untracked().use_default_accent_color
                    attr:name="preferences[use_default_accent_color]"
                    attr:id="use-default-color"
                    on:change=on_default_checked
                />
            </div>
            <label for="accent-color" class="title" style:grid-column=1>
                Accent Color
            </label>
            <div style:grid-column=2>
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
            </div>

            <BoolOption
                label="Show Body Border".to_string()
                checked=preferences.get_untracked().show_body_border
                attr:name="preferences[show_body_border]"
                attr:id="show-body-border"
                on:change=handle_border_change
            />

            <SaveOnPause />
        </>
    }
}

#[component]
pub fn AccountPreferences() -> impl IntoView {
    view! {
        <span for="change-username" class="title" style:grid-column=1>
            Change Username
        </span>
        <div style:grid-column=2>
            <A class=style::edit href="/change-username">
                <i class="fa-solid fa-arrow-right"></i>
            </A>
        </div>

        <span class="title" style:grid-column=1 style:grid-column=1>
            Change Password
        </span>
        <div style:grid-column=2>
            <A class=style::edit href="/change-password">
                <i class="fa-solid fa-arrow-right"></i>
            </A>
        </div>
    }
}

#[component]
pub fn MiscPreferences() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();

    let on_separator_checked =
        move |_: Event| preferences.update(|p| p.show_separator = !p.show_separator);

    let on_multi_checked = move |_: Event| preferences.update(|p| p.multi_select = !p.multi_select);

    view! {
        <label for="show-separator" class="title" style:grid-column=1>
            Show Treeview Separator
        </label>
        <div style:grid-column=2>
            <Slider
                checked=preferences.get_untracked().show_separator
                attr:name="preferences[show_separator]"
                attr:id="show-separator"
                on:change=on_separator_checked
            />
        </div>

        <label for="multi-select" class="title" style:grid-column=1>
            Use Multi Select (experimental)
        </label>
        <div style:grid-column=2>
            <Slider
                checked=preferences.get_untracked().multi_select
                attr:name="preferences[multi_select]"
                attr:id="multi-select"
                on:change=on_multi_checked
            />
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
        <label for="save-on-pause" style:grid-column=1>
            Save on pause
        </label>
        <div style:grid-column=2>
            <Slider
                checked
                attr:name="preferences[save_on_pause]"
                attr:id="save-on-pause"
                on:change=on_change
            />
        </div>
    }
}

#[component]
fn BoolOption(
    #[prop(into, optional)] label: Option<MaybeSignal<String>>,
    #[prop(into)] checked: MaybeSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let id = StoredValue::new(
        attrs
            .iter()
            .find(|item| item.0 == "id")
            .map(|item| item.1.clone()),
    );

    let label = Signal::derive(move || label.as_ref().map(|item| item()));

    view! {
        <Show when=move || label().is_some()>
            <label
                for=move || {
                    id.get_value().map(|attr| attr.as_nameless_value_string()).unwrap_or_default()
                }
                style:grid-column=1
            >
                {label().unwrap()}
            </label>
        </Show>
        <div style:grid-column=2>
            <Slider checked {..attrs} />
        </div>
    }
}

#[component]
fn ListViewRow(name: &'static str) -> impl IntoView {
    view! {
        <A href=|| name.to_string()>
            <div class=stylance::classes!(style::row, style::selectable)>
                <div class=style::row_body>
                    <span>{name}</span>
                </div>
            </div>
        </A>
    }
}
