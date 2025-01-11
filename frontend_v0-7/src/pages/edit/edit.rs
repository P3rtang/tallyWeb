use super::*;

stylance::import_style!(
    #[allow(dead_code)]
    style,
    "./edit.module.scss"
);

#[component]
pub fn EditWindow() -> impl IntoView {
    // let store = expect_context::<RwSignal<CountableStore>>();

    // let hide_border = create_read_slice(preferences, |p| !p.show_body_border);

    // we need to render the outlet first since it sets the selection key from the url
    // let outlet_view = move || view! { <Outlet /> };

    // let (width, set_width) = signal(400);

    view! {
        <Page>
            <PageContent hide_border=true slot>
                <EditCountableWindow />
            </PageContent>
            <PageSidebar is_shown=true slot>
            <div/>
            </PageSidebar>
            <PageNavbar slot>
                <Navbar />
            </PageNavbar>
        </Page>
    }
}

#[derive(Params, Clone, PartialEq, Default)]
struct Selection {
    slct: CountableId,
}

#[component]
pub fn EditCountableWindow() -> impl IntoView {
    let params = use_query::<Selection>();
    let key = move || params.get().map(|s| s.slct);

    leptos::logging::log!("{:?}", key());

    view! {
        <div class=style::form>
            <Show when=move || key().is_ok()>
                <EditCounterBox key=Signal::derive(move || key().unwrap_or_default()) />
            </Show>
        </div>
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
    // let msg = expect_context::<MessageJar>();
    let action = ServerAction::<api::EditCountableForm>::new();

    let referer = use_referer(Default::default());
    let navigate = use_navigate();

    let kind = create_read_slice(store, move |s| s.kind(&key().into()));

    let params = use_params::<UserName>();
    let user_name = move || params.get().map(|p| p.id).ok();

    Effect::new(move |_| match action.value()() {
        Some(Ok(_)) => {
            if let Some(referer) = referer.get() {
                navigate(&referer, Default::default())
            } else {
                navigate(
                    format!("/{}", user_name().unwrap_or_default()).as_str(),
                    Default::default(),
                )
            }
        }
        Some(Err(err)) => {
            // match err {
            //     ServerFnError::WrappedServerError(err) => msg.set_err(err),
            //     ServerFnError::Registration(err) => msg.set_err(err),
            //     ServerFnError::Request(_) => msg.set_err("Could not reach server"),
            //     ServerFnError::Response(err) => msg.set_err(err),
            //     ServerFnError::ServerError(err) => msg.set_err(err),
            //     ServerFnError::Deserialization(err) => msg.set_err(err),
            //     ServerFnError::Serialization(err) => msg.set_err(err),
            //     ServerFnError::Args(err) => msg.set_err(err),
            //     ServerFnError::MissingArg(err) => msg.set_err(err),
            // };
        }
        None => {}
    });

    // Effect::new(move |_| {
    //     if let Some(Ok(_)) = action.value()() {
    //         rs.refetch();
    //         use_navigate()(format!("/{}", key()).as_str(), Default::default())
    //     }
    // });

    // let undo = move |_| {
    //     rs.refetch();
    // };

    view! {
        <ActionForm action>
            <SessionFormInput session />
            <input type="hidden" name="countable_key" value=move || key.get().0.to_string() />
            <input type="hidden" name="countable_kind" value=move || kind().to_string() />
            <table class=style::content>
                <tbody>
                    <tr class=stylance::classes!(style::row, style::text_row)>
                        <EditName key />
                    </tr>
                    <tr class=stylance::classes!(style::row, style::text_row)>
                        <EditCount key />
                    </tr>
                    <tr class=stylance::classes!(style::row, style::text_row)>
                        <EditStepSize key />
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
            <action-buttons class=style::action_buttons>
                <action-start></action-start>
                <action-end>
                    <button type="button" class="hover-darken">
                        <div>Undo</div>
                    </button>
                    <button type="submit" class=stylance::classes!(style::confirm, "hover-darken")>
                        <div>Submit</div>
                    </button>
                </action-end>
            </action-buttons>
        </ActionForm>
    }
}

#[component]
fn EditName(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let (name, set_name) = create_slice(
        store,
        move |s| s.name(&key().into()),
        move |s, name: String| s.set_name(&key.get(), name),
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
fn EditCount(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
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
fn EditStepSize(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let step = create_read_slice(store, move |s| s.recursive_ref().step_size(&key().into()));

    view! {
        <td>
            <label for="change-step">Step size</label>
        </td>
        <td>
            <div class=style::boxed>
                <input
                    type="number"
                    value=step
                    prop:value=step
                    id="change-step"
                    name="countable_step"
                    style:text-align="end"
                />
            </div>
        </td>
    }
}

#[component]
fn EditTime(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let time = create_read_slice(store, move |s| s.recursive_ref().time(&key().into()));

    let hour_ref = NodeRef::<html::Input>::new();
    let min_ref = NodeRef::<html::Input>::new();
    let sec_ref = NodeRef::<html::Input>::new();
    let millis_ref = NodeRef::<html::Input>::new();

    let limit_num = |ev: ev::Event, node_ref: NodeRef<html::Input>, min, max| {
        if let Some(node) = node_ref.get() {
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
        if let Some(node) = node_ref.get() {
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
fn EditHunttype(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
    let store = expect_context::<RwSignal<CountableStore>>();
    let hunt_type = move || store().recursive_ref().hunttype(&key().into());
    let selected = Memo::new(move |_| hunt_type().into());

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
                    attr:value=move || hunt_type().as_str()
                    selected
                    options
                />
            </div>
        </td>
    }
}

#[component]
fn EditCharm(#[prop(into)] key: Signal<CountableId>) -> impl IntoView {
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
