use futures::FutureExt;
use leptos::{tachys::html::node_ref::node_ref, task::spawn_local};
use web_sys::SubmitEvent;

use super::*;

#[component]
pub fn EditWindow() -> impl IntoView {
    let sidebar = expect_context::<page_context::PageContext>().sidebar;

    let sidebar_width = sidebar.width();
    let set_width = move |w| sidebar.set_width().set(w);

    let params = use_query::<Selection>();
    let selection = Memo::new(move |_| params.get().unwrap_or_default());
    provide_context(selection);

    let content_attr =
        view! { <{..} style:max-width="1200px" style:width="100%" style:margin="auto" /> }
            .into_attr_fn();

    view! {
        <Page>
            <PageContent attrs=content_attr hide_border=true slot>
                <EditCountableWindow />
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
fn SidebarContent() -> impl IntoView {
    let screen = hooks::use_screen();
    let sidebar = expect_context::<page_context::PageContext>().sidebar;
    let store = expect_context::<RwSignal<CountableStore>>();
    let selection = expect_context::<Memo<Selection>>();

    let each = move || {
        let mut root = store.get().root_node_ids();
        root.sort_by_key(|a| store.get().name(a));
        root
    };

    let is_selected = move |key: CountableId| selection.get().slct == key;

    view! {
        <div style:width=sidebar.width_attr() class=style::sidebar>
            <nav class=main::navbar>
                <Show when=move || {
                    screen.get().viewport() <= ViewPort::Small
                }>{sidebar.toggle_button()}</Show>
            </nav>
            <div class=style::content>
                <List each key=|c| *c children=move |c| store.get().children(&c)>
                    <RowSlot is_selected let:child slot>
                        <TreeRow countable=child />
                    </RowSlot>
                    <Separator slot>
                        <hr />
                    </Separator>
                </List>
            </div>
        </div>
    }
}

#[component]
fn TreeRow(countable: CountableId) -> AnyView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let href = move || format!("?slct={}", countable.0);

    let name = Signal::derive(move || store.get().name(&countable));

    view! {
        <A href style:width="100%">
            {name}
        </A>
    }
    .into_any()
}

#[derive(Params, Clone, PartialEq, Default)]
struct Selection {
    slct: CountableId,
}

#[component]
pub fn EditCountableWindow() -> impl IntoView {
    let params = use_query::<Selection>();
    let key = Memo::new(move |_| params.get().map(|s| s.slct));

    view! {
        <Show when=move || key.get().is_ok()>
            <EditCounterBox key=Signal::derive(move || key.get().unwrap_or_default()) />
        </Show>
    }
}

#[derive(Debug, Clone, Params, PartialEq, Eq, Default)]
struct Key {
    key: String,
}

#[component]
fn EditCounterBox(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let session = expect_context::<RwSignal<UserSession>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let action = ServerAction::<api::EditCountableForm>::new();
    let store_resc = expect_context::<Resource<Option<CountableStore>>>();
    let local_store_resc = expect_context::<Resource<Option<CountableStore>>>();

    let history = use_history();
    let navigate = use_navigate();
    let message = use_message();

    let handle_error = move |err| {
        message.server_err(err);
    };

    let kind = Signal::derive(move || store.get().kind(&key.get()));

    let params = use_params::<UserName>();
    let user_name = move || params.get().map(|p| p.id).ok();

    let close_href = StoredValue::new(history.back().get_untracked().map(|url| url.to_string()));

    // TODO: use a server side redirection instead, passed as a form argument
    Effect::new(move |_| match action.value().get() {
        Some(Ok(_)) => {
            // TODO: maybe instead of refetching I could have the return set the store state
            store_resc.refetch();
            navigate(
                &close_href.get_value().unwrap_or("/".to_string()),
                Default::default(),
            );
        }
        Some(Err(err)) => handle_error(err),
        None => {}
    });

    let on_undo = move |_| local_store_resc.refetch();

    let title = Signal::derive(move || store.get().name(&key.get()));

    view! {
        <Form action on_undo attr:id="edit-form">
            <HeaderSlot
                title
                on_close=on_undo
                close_href=close_href.get_value().map(Signal::from)
                slot
            >
                <DeleteButton key />
            </HeaderSlot>

            <SessionFormInput session />
            <input type="hidden" name="countable[key]" value=move || key.get().0.to_string() />
            <input type="hidden" name="countable[kind]" value=move || kind().to_string() />
            <EditName key />
            <EditCount key />
            <EditStepSize key />
            <EditTime key />
            <EditHunttype key />
            <EditCharm key />
        </Form>
    }
}

#[component]
fn DeleteButton(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let session = expect_context::<RwSignal<UserSession>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let saving = hooks::use_local_saving::<CountableStore>();
    let confirm = use_confirm();
    let form_ref = NodeRef::<leptos::html::Form>::new();
    let redirect = move || format!("/{}", session.get().username);

    let kind = Signal::derive(move || store.get().kind(&key.get()));

    let delete_action = ServerAction::<api::ArchiveCountable>::new();

    Effect::new(move |_| match delete_action.value().get() {
        Some(Ok(())) => store.update(|s| {
            s.archive(&key.get());

            if let Some(func) = saving.clone() {
                func(s.clone())
            }
        }),
        Some(Err(_err)) => (),
        None => (),
    });

    let handle_submit = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        spawn_local(
            confirm("Delete this counter".to_string()).then(async move |ok| {
                if ok {
                    form_ref.get_untracked().unwrap().submit();
                }
            }),
        );
    };

    view! {
        <ActionForm
            action=delete_action
            node_ref=form_ref
            attr:id="delete-form"
            on:submit=|ev| ev.prevent_default()
        >
            <input type="hidden" name="id" value=move || key.get().0.to_string() />
            <input type="hidden" name="kind" value=move || kind.get().to_string() />
            <input type="hidden" name="redirect" value=redirect />
            <button
                class="hover-darken icon"
                aria_label="delete countable"
                form="delete-form"
                on:click=handle_submit
            >
                <div>
                    <Icon kind=IconKind::TrashCan />
                </div>
            </button>
        </ActionForm>
    }
}

#[component]
fn EditName(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let (name, set_name) = create_slice(
        store,
        move |s| s.name(&key.get()),
        move |s, name: String| s.set_name(&key.get(), name),
    );

    let on_input = move |ev| set_name(event_target_value(&ev));

    view! {
        <TextField
            id="change-name"
            label="Name"
            prop:value=name
            attr:value=name
            attr:name="countable[name]"
            attr:placeholder="Name"
            on:input=on_input
        />
    }
}

#[component]
fn EditCount(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let count = Signal::derive(move || store.get().recursive_ref().count(&key.get()));

    view! {
        <TextField
            id="change-count"
            label="Count"
            type_="number"
            prop:value=count
            attr:value=count
            attr:name="countable[count]"
        />
    }
}

#[component]
fn EditStepSize(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let step = Signal::derive(move || store.get().recursive_ref().step_size(&key.get()));

    view! {
        <TextField
            id="change-step"
            label="Step size"
            type_="number"
            prop:value=step
            attr:value=step
            attr:name="countable[step]"
        />
    }
}

#[component]
fn EditTime(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();

    let (time, set_time) = signal(
        store
            .get_untracked()
            .recursive_ref()
            .time(&key.get_untracked()),
    );

    let time_memo = Memo::new(move |prev| {
        if prev.is_some_and(|(k, t)| *k != key.get()) {
            set_time(store.get().recursive_ref().time(&key.get()));
        }

        (key.get(), time.get())
    });

    view! {
        <TimeDeltaField
            label="Elapsed Time"
            value=Signal::derive(move || time_memo.get().1)
            on_change=move |t| set_time.set(t)
            name="countable[time]"
            id="change-time"
            use_single_form_value=true
        />
    }
}

#[component]
fn EditHunttype(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();

    let (hunttype, set_hunttype) = create_slice(
        store,
        move |s| s.recursive_ref().hunttype(&key.get()),
        move |s, ht| s.recursive_ref().set_hunttype(&key.get(), ht),
    );

    let options = vec![
        Hunttype::OldOdds,
        Hunttype::NewOdds,
        Hunttype::Masuda(Masuda::GenIV),
        Hunttype::Masuda(Masuda::GenV),
        Hunttype::Masuda(Masuda::GenVI),
        Hunttype::SOS,
        // hunt_option(Hunttype::DexNav),
    ];

    view! {
        <SelectField
            label="Hunt method"
            id="change-hunttype"
            attrs=move || view! { <{..} attr:name="countable[hunttype]" /> }
            value=hunttype
            on_change=move |ht| {
                if let Some(ht) = ht {
                    set_hunttype.set(ht)
                }
            }
            options
        />
    }
}

#[component]
fn EditCharm(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let checked = Signal::derive(move || store.get().recursive_ref().has_charm(&key.get()));

    view! {
        <BoolField
            id="has-charm"
            label="Has Charm"
            prop:checked=checked
            attr:checked=checked
            attr:name="countable[charm]"
        />
    }
}
