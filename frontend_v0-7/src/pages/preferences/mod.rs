use super::*;

stylance::import_style!(style, "./prefs.module.scss");

#[derive(Debug, Clone, Default, Params, PartialEq)]
pub struct Topic {
    topic: Option<String>,
}

impl PartialEq<&'static str> for Topic {
    fn eq(&self, other: &&'static str) -> bool {
        self.topic.as_ref().is_some_and(|t| t == *other)
    }
}

#[component]
pub fn PrefsWindow() -> impl IntoView {
    let params = use_query::<Topic>();
    let topic = Memo::new(move |_| params.get().unwrap_or_default());
    provide_context(topic);

    let sidebar = expect_context::<page_context::PageContext>().sidebar;
    let set_width = move |w| sidebar.set_width().set(w);

    view! {
        <Page>
            <PageContent hide_border=true slot>
                <PrefsContent />
            </PageContent>
            <PageSidebar width=sidebar.width() on_resize=set_width is_shown=sidebar.is_shown() slot>
                <SidebarContent />
            </PageSidebar>
            <PageNavbar slot>
                <Navbar />
            </PageNavbar>
        </Page>
    }
}

#[component]
pub fn SidebarContent() -> impl IntoView {
    let screen = hooks::use_screen();
    let sidebar = expect_context::<page_context::PageContext>().sidebar;
    let topic = expect_context::<Memo<Topic>>();

    let is_selected = move |t: &'static str| topic.get().topic.is_some_and(|topic| topic == *t);

    view! {
        <div style:width=sidebar.width_attr() style:max-width="100vw">
            <nav class=main::navbar>
                <Show when=move || screen.get().viewport() <= ViewPort::Small>
                {sidebar.toggle_button()}
                </Show>
            </nav>
            <List
                each=move || vec!["styling", "account", "misc"]
                key=move |topic| *topic
            >
                <RowSlot is_selected let:topic slot>
                    <TreeRow topic />
                </RowSlot>
            </List>
        </div>
    }
}

#[component]
fn TreeRow(#[prop(into)] topic: Signal<String>) -> AnyView {
    let href = move || format!("?topic={}", topic.get());

    view! { <A href style:width="100%">{topic}</A> }.into_any()
}

#[component]
fn PrefsContent() -> impl IntoView {
    let action = ServerAction::<api::SavePreferences>::new();
    let session = expect_context::<RwSignal<UserSession>>();

    let referer = hooks::use_referer(Default::default());
    let close_href = Signal::derive(move || {
        referer
            .get()
            .unwrap_or(format!("/{}", session.get().username))
    });

    view! {
        <Form action>
            <HeaderSlot title="Preferences" close_href slot />
            <session::SessionFormInput session />

            <AccentColor />
            <UseDefaultAccentColor />
            <ShowBodyBorder />
            <ShowSeparator />
            <SaveOnPause />
        </Form>
    }
}

#[component]
fn AccentColor() -> impl IntoView {
    let prefs = expect_context::<Resource<Preferences>>();

    let topic = expect_context::<Memo<Topic>>();

    let accent_color = move || {
        prefs
            .get()
            .map(|p| p.accent_color.to_string())
            .unwrap_or_default()
    };

    let on_change = move |ev: ev::Event| {
        let color = event_target_value(&ev);
        if color.is_empty() {
            ev.prevent_default()
        }

        match color.as_str().try_into() {
            Ok(c) => prefs.update(|p| {
                if let Some(p) = p {
                    p.accent_color = c
                }
            }),
            Err(err) => warn!("{}", err),
        }
    };

    let fallback = move || {
        view! {
            <input type="hidden" name="preferences[accent_color]" value=accent_color />
        }
    };

    view! {
        <Show when=move || topic.get() == "styling" fallback=fallback>
            <ColorField
                id="accent-color"
                label="Accent colour"
                on:input=on_change
                prop:value=accent_color
                attr:value=accent_color
                attr:name="preferences[accent_color]"
                attr:disabled=move || prefs.get().is_some_and(|p| p.use_default_accent_color)
            />
        </Show>
    }
}

#[component]
fn UseDefaultAccentColor() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();
    let topic = expect_context::<Memo<Topic>>();

    let (use_default, set_use_default) = create_slice(
        preferences,
        move |p| p.use_default_accent_color,
        move |p, b| p.use_default_accent_color = b,
    );

    let handle_change = move |_: ev::Event| set_use_default.set(!use_default.get());

    let fallback = move || {
        view! {
            <input type="hidden" name="preferences[use_default_accent_color]" prop:value=use_default />
        }
    };

    view! {
        <Show when=move || topic.get() == "styling" fallback=fallback>
            <BoolField
                id="accent-color"
                label="Use default accent color"
                on:change=handle_change
                prop:checked=use_default
                attr:checked=use_default
                attr:name="preferences[use_default_accent_color]"
            />
        </Show>
    }
}

#[component]
fn ShowBodyBorder() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();
    let topic = expect_context::<Memo<Topic>>();

    let (show_border, set_show_border) = create_slice(
        preferences,
        move |p| p.show_body_border,
        move |p, b| p.show_body_border = b,
    );

    let handle_change = move |_: ev::Event| set_show_border.set(!show_border.get());

    let fallback = move || {
        view! {
            <input type="hidden" name="preferences[show_body_border]" prop:value=show_border />
        }
    };
    view! {
        <Show when=move || topic.get() == "styling" fallback=fallback>
            <BoolField
                id="show-body-border"
                label="Show body border"
                on:change=handle_change
                prop:checked=show_border
                attr:checked=show_border
                attr:name="preferences[show_body_border]"
            />
        </Show>
    }
}

#[component]
fn ShowSeparator() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();
    let topic = expect_context::<Memo<Topic>>();

    let (show_separator, set_show_separator) = create_slice(
        preferences,
        move |p| p.show_separator,
        move |p, b| p.show_separator = b,
    );

    let handle_change = move |_: ev::Event| set_show_separator.set(!show_separator.get());

    let fallback = move || {
        view! {
            <input type="hidden" name="preferences[show_separator]" prop:value=show_separator />
        }
    };
    view! {
        <Show when=move || topic.get() == "styling" fallback=fallback>
            <BoolField
                id="show-separator"
                label="Show treeview separator"
                on:change=handle_change
                prop:checked=show_separator
                attr:checked=show_separator
                attr:name="preferences[show_separator]"
            />
        </Show>
    }
}

#[component]
fn SaveOnPause() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();
    let topic = expect_context::<Memo<Topic>>();

    let (save_on_pause, set_save_on_pause) = create_slice(
        preferences,
        move |p| p.save_on_pause,
        move |p, b| p.save_on_pause = b,
    );

    let handle_change = move |_: ev::Event| set_save_on_pause.set(!save_on_pause.get());

    let fallback = move || {
        view! {
            <input type="hidden" name="preferences[save_on_pause]" prop:value=save_on_pause />
        }
    };
    view! {
        <Show when=move || topic.get() == "misc" fallback=fallback>
            // TODO: add a tooltip to explain this is to combat waterfalls when pressing pause a lot
            <BoolField
                id="save-on-pause"
                label="Autosave on pause"
                on:change=handle_change
                prop:checked=save_on_pause
                attr:checked=save_on_pause
                attr:name="preferences[save_on_pause]"
            />
        </Show>
    }
}
