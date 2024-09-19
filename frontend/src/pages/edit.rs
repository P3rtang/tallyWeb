#![allow(non_snake_case)]
use components::{
    MessageJar, Select, SelectionModel, ShowSidebar, Sidebar, SidebarLayout, TreeViewWidget,
};
use elements::{Navbar, SortMethod, SortSearch};
use leptos::*;
use leptos_router::{use_params, ActionForm, Outlet, Params, A};

use super::*;

stylance::import_style!(
    #[allow(dead_code)]
    style,
    "../../style/edit.module.scss"
);

#[component]
pub fn EditWindow() -> impl IntoView {
    let preferences = expect_context::<RwSignal<Preferences>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let screen = expect_context::<Screen>();
    let sort_method = expect_context::<RwSignal<SortMethod>>();

    let sidebar_layout: Signal<SidebarLayout> = create_read_slice(screen.style, |s| (*s).into());

    let selection = create_rw_signal(SelectionModel::<uuid::Uuid, Countable>::new());
    provide_context(selection);

    let width = create_rw_signal(400);
    let show_sort_search = create_rw_signal(true);
    let show_sep = create_read_slice(preferences, |pref| pref.show_separator);

    let show_sidebar = create_rw_signal(ShowSidebar(false));

    let min_height = create_memo(move |_| match (screen.style)() {
        ScreenStyle::Portrait => Some("110vh"),
        ScreenStyle::Small => None,
        ScreenStyle::Big => None,
    });

    // we need to render the outlet first since it sets the selection key from the url
    let outlet_view = view! { <Outlet /> };

    let sidebar_update_memo =
        create_memo(move |_| ((screen.style)(), selection().get_owned_selected_keys()));

    create_isomorphic_effect(move |_| match sidebar_update_memo.get() {
        (ScreenStyle::Portrait, e) => {
            width.set(0);
            logging::log!("{}", e.is_empty());
            show_sidebar.set(ShowSidebar(e.is_empty()));
        }
        (ScreenStyle::Small, e) => {
            width.set(0);
            show_sidebar.set(ShowSidebar(e.is_empty()));
        }
        (ScreenStyle::Big, _) => {
            width.set(400);
            show_sidebar.set(ShowSidebar(true));
        }
    });

    let each_child = move |countable: &Countable| {
        let mut children = store().children(&countable.uuid().into());
        children.sort_by(|a, b| sort_method().sort_by()(&store.get_untracked(), a, b));
        children
            .into_iter()
            .map(|c| store.get_untracked().get(&c))
            .collect::<Option<Vec<_>>>()
            .unwrap_or_default()
    };

    view! {
        <div style:display="flex">
            <Sidebar display=show_sidebar layout=sidebar_layout width>
                <nav>
                    <SortSearch
                        shown=show_sort_search
                        search=create_rw_signal(String::new())
                        on_keydown=|_| ()
                    />
                </nav>
                <TreeViewWidget
                    each=move || {
                        let mut root_nodes = store().filter(|c| !c.is_archived()).root_nodes();
                        root_nodes
                            .sort_by(|a, b| sort_method()
                                .sort_by()(
                                &store.get_untracked(),
                                &a.uuid().into(),
                                &b.uuid().into(),
                            ));
                        root_nodes
                    }

                    key=|countable| countable.uuid()
                    each_child
                    view=|countable| view! { <TreeViewRow key=countable.uuid() /> }
                    show_separator=show_sep
                    selection_model=selection
                />
            </Sidebar>
            <section
                style:width=move || format!("calc(100vw - {}px)", width())
                style:min-height=min_height
            >
                <Navbar show_sidebar />
                {outlet_view}
            </section>
        </div>
    }
}

#[component]
fn TreeViewRow(key: uuid::Uuid) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let name = move || store().name(&key.into());

    view! {
        <A href=move || key.to_string()>
            <div class=style::tree_row>
                <span>{name}</span>
            </div>
        </A>
    }
}

#[component]
pub fn EditCountableWindow() -> impl IntoView {
    let selection = expect_context::<SelectionSignal>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let screen = expect_context::<Screen>();

    let key_memo = create_memo(move |old_key| {
        let new_key = use_params::<Key>()()
            .ok()
            .and_then(|p| uuid::Uuid::parse_str(&p.key).ok());

        if let Some(key) = new_key
            && new_key != old_key.copied()
        {
            selection.update(|sel| sel.select(&key))
        }

        new_key.unwrap_or_default()
    });

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

    let valid = move || store().contains(&key_memo().into());

    view! {
        <h1 style:color="white" style:padding="12px 48px">
            Edit
        </h1>
        <div style:display="flex" style:justify-content="center">
            <Show when=valid>
                <edit-form class=form_style>
                    <EditCounterBox key=key_memo />
                </edit-form>
            </Show>
        </div>
    }
}

#[derive(Debug, Clone, Params, PartialEq, Eq, Default)]
struct Key {
    key: String,
}

#[component]
fn EditCounterBox(#[prop(into)] key: MaybeSignal<uuid::Uuid>) -> impl IntoView {
    let rs = expect_context::<StateResource>();
    let session = expect_context::<RwSignal<UserSession>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let msg = expect_context::<MessageJar>();
    let action = create_server_action::<api::EditCountableForm>();
    let screen = expect_context::<Screen>();

    let kind = create_read_slice(store, move |s| s.kind(&key().into()));

    create_effect(move |_| match action.value()() {
        Some(Ok(_)) => {
            leptos_router::use_navigate()(format!("/{}", key()).as_str(), Default::default())
        }
        Some(Err(err)) => {
            match err {
                ServerFnError::WrappedServerError(err) => msg.set_err(err),
                ServerFnError::Registration(err) => msg.set_err(err),
                ServerFnError::Request(_) => msg.set_err("Could not reach server"),
                ServerFnError::Response(err) => msg.set_err(err),
                ServerFnError::ServerError(err) => msg.set_err(err),
                ServerFnError::Deserialization(err) => msg.set_err(err),
                ServerFnError::Serialization(err) => msg.set_err(err),
                ServerFnError::Args(err) => msg.set_err(err),
                ServerFnError::MissingArg(err) => msg.set_err(err),
            };
        }
        None => {}
    });

    create_effect(move |_| {
        if let Some(Ok(_)) = action.value()() {
            rs.refetch();
            leptos_router::use_navigate()(format!("/{}", key()).as_str(), Default::default())
        }
    });

    let undo = move |_| {
        rs.refetch();
    };

    view! {
        <ActionForm action>
            <SessionFormInput session />
            <input type="hidden" name="countable_key" value=move || key().to_string() />
            <input type="hidden" name="countable_kind" value=move || kind().to_string() />
            <table style:display="flex" style:flex-flow="column" class=style::content>
                <tbody>
                    <tr class=stylance::classes!(style::row, style::text_row)>
                        <EditName key />
                    </tr>
                    <tr class=stylance::classes!(style::row, style::text_row)>
                        <EditCount key />
                    </tr>
                    <tr class=stylance::classes!(style::row, style::text_row)>
                        <EditTime key />
                    </tr>
                    <tr class=stylance::classes!(style::row, style::text_row)>
                        <EditHunttype key />
                    </tr>
                    <tr class=style::row>
                        <EditCharm key />
                    </tr>
                </tbody>
            </table>
            <action-buttons class=move || {
                stylance::classes!(
                    style::action_buttons, match (screen.style) () { ScreenStyle::Portrait =>
                    Some(style::fixed), ScreenStyle::Small => None, ScreenStyle::Big => None, }
                )
            }>

                <action-start></action-start>
                <action-end>
                    <button type="button" on:click=undo>
                        Undo
                    </button>
                    <button type="submit" class=style::confirm>
                        Submit
                    </button>
                </action-end>
            </action-buttons>
        </ActionForm>
    }
}

#[component]
fn EditName(#[prop(into)] key: MaybeSignal<uuid::Uuid>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let (name, set_name) = create_slice(
        store,
        move |s| s.name(&key().into()),
        move |s, name: String| s.set_name(&key().into(), &name),
    );

    let on_input = move |ev| set_name(event_target_value(&ev));

    view! {
        <td>
            <label for="change-name">Name</label>
        </td>
        <td>
            <div class=style::boxed>
                <input
                    type="text"
                    value=name
                    prop:value=name
                    id="change-name"
                    name="countable_name"
                    on:input=on_input
                    style:text-align="end"
                />
            </div>
        </td>
    }
}

#[component]
fn EditCount(#[prop(into)] key: MaybeSignal<uuid::Uuid>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let count = create_read_slice(store, move |s| s.recursive_ref().count(&key().into()));

    view! {
        <td>
            <label for="change-count">Count</label>
        </td>
        <td>
            <div class=style::boxed>
                <input
                    type="number"
                    value=count
                    prop:value=count
                    id="change-count"
                    name="countable_count"
                    style:text-align="end"
                />
            </div>
        </td>
    }
}

#[component]
fn EditTime(#[prop(into)] key: MaybeSignal<uuid::Uuid>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let time = create_read_slice(store, move |s| s.recursive_ref().time(&key().into()));

    let hour_ref = create_node_ref::<html::Input>();
    let min_ref = create_node_ref::<html::Input>();
    let sec_ref = create_node_ref::<html::Input>();
    let millis_ref = create_node_ref::<html::Input>();

    let limit_num = |ev: ev::Event, node_ref: NodeRef<html::Input>, min, max| {
        if let Some(node) = node_ref() {
            let mut new_val = event_target_value(&ev);
            if new_val.parse::<i64>().is_ok_and(|v| min <= v && v < max) {
            } else if !new_val.is_empty() {
                new_val.remove(new_val.len() - 1);
                node.set_value(&new_val);
            }
        }
    };

    let pad_hours = move || format!("{:02}", time().num_hours());
    let pad_mins = move || format!("{:02}", time().num_minutes() % 60);
    let pad_secs = move || format!("{:02}", time().num_seconds() % 60);
    let pad_millis = move || format!("{:03}", time().num_milliseconds() % 1000);

    let pad_input = move |node_ref: NodeRef<html::Input>, w| {
        // we check whether a signal is disposed so we know the node_ref is disposed as well
        if time.try_get().is_none() {
            return;
        }
        if let Some(node) = node_ref() {
            if let Ok(num) = node.value().parse::<i32>() {
                node.set_value(format!("{:0w$}", num, w = w).as_str());
            } else if node.value() == "" {
                node.set_value("0".repeat(w).as_str())
            }
        }
    };

    view! {
        <td>Time</td>
        <td>
            <div class=style::boxed style:text-align="end">
                <label for="change-hours">
                    <input
                        type="number"
                        value=pad_hours
                        id="change-hours"
                        name="countable_hours"
                        style:width="4ch"
                        style:text-align="end"
                        node_ref=hour_ref
                        on:focusout=move |_| pad_input(hour_ref, 2)
                    />
                    :
                </label>
                <label for="change-mins">
                    <input
                        type="number"
                        value=pad_mins
                        id="change-mins"
                        name="countable_mins"
                        max="59"
                        style:width="2ch"
                        style:text-align="end"
                        node_ref=min_ref
                        on:input=move |ev| limit_num(ev, min_ref, 0, 59)
                        on:focusout=move |_| pad_input(min_ref, 2)
                    />
                    :
                </label>
                <label for="change-secs">
                    <input
                        type="number"
                        value=pad_secs
                        id="change-secs"
                        name="countable_secs"
                        max="59"
                        style:width="2ch"
                        style:text-align="end"
                        node_ref=sec_ref
                        on:input=move |ev| limit_num(ev, sec_ref, 0, 59)
                        on:focusout=move |_| pad_input(sec_ref, 2)
                    />
                    .
                </label>
                <label for="change-millis">
                    <input
                        type="number"
                        value=pad_millis
                        id="change-millis"
                        name="countable_millis"
                        max="999"
                        style:width="3ch"
                        style:text-align="end"
                        node_ref=millis_ref
                        on:input=move |ev| limit_num(ev, millis_ref, 0, 999)
                        on:focusout=move |_| pad_input(millis_ref, 3)
                    />
                </label>
            </div>
        </td>
    }
}

#[component]
fn EditHunttype(#[prop(into)] key: MaybeSignal<uuid::Uuid>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let hunt_type = move || store().hunttype(&key().into());
    let selected = create_memo(move |_| hunt_type().into());

    let hunt_option = |ht: Hunttype| -> (&'static str, &'static str) { (ht.repr(), ht.into()) };

    let options = vec![
        hunt_option(Hunttype::OldOdds).into(),
        hunt_option(Hunttype::NewOdds).into(),
        hunt_option(Hunttype::Masuda(Masuda::GenIV)).into(),
        hunt_option(Hunttype::Masuda(Masuda::GenV)).into(),
        hunt_option(Hunttype::Masuda(Masuda::GenVI)).into(),
        hunt_option(Hunttype::SOS).into(),
        // hunt_option(Hunttype::DexNav).into(),
    ];

    view! {
        <td>
            <label for="change-hunttype">Method</label>
        </td>
        <td style:text-align="start">
            <div class=style::boxed>
                <Select
                    attr:id="change-hunttype"
                    attr:name="countable_hunttype"
                    attr:value=hunt_type
                    selected
                    options
                />
            </div>
        </td>
    }
}

#[component]
fn EditCharm(#[prop(into)] key: MaybeSignal<uuid::Uuid>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let checked = create_read_slice(store, move |s| s.has_charm(&key().into()));

    view! {
        <td>
            <label for="has-charm">Has Charm</label>
        </td>
        <td>
            <components::Slider
                attr:id="has-charm"
                attr:name="countable_charm"
                checked
            ></components::Slider>
        </td>
    }
}
