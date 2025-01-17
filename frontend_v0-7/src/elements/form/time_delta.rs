use super::*;

stylance::import_style!(style, "./form.module.scss");

// TODO: log the error with the msg system
/**
 * [OnChange] callback for [TimeDeltaField]
 *
 * # params
 *   [TimeDelta] - The new time value of the [TimeDeltaField]
 *
 * # return
 *   [AppResult]
 *   - `Ok(())` - The new value is accepted
 *   - `Err(`[AppError]`)` - The old value will be reinstated and the error logged
 */
pub struct OnChange(Arc<dyn Fn(TimeDelta) -> AppResult<()>>);

impl Default for OnChange {
    fn default() -> Self {
        Self(Arc::new(move |_| Ok(())))
    }
}

impl<F: Fn(TimeDelta) -> AppResult<()> + 'static> From<F> for OnChange {
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

#[derive(Clone)]
#[slot]
pub struct DeltaHour {
    attrs: Arc<dyn Fn() -> AnyAttribute + Send + Sync + 'static>,
}

#[derive(Clone)]
#[slot]
pub struct DeltaMinute {
    attrs: Arc<dyn Fn() -> AnyAttribute + Send + Sync + 'static>,
}

#[derive(Clone)]
#[slot]
pub struct DeltaSecond {
    attrs: Arc<dyn Fn() -> AnyAttribute + Send + Sync + 'static>,
}

#[derive(Clone)]
#[slot]
pub struct DeltaMilli {
    attrs: Arc<dyn Fn() -> AnyAttribute + Send + Sync + 'static>,
}

#[component]
pub fn TimeDeltaField(
    #[prop(into, optional)] label: Option<Signal<String>>,
    #[prop(into, optional)] name: Signal<String>,
    #[prop(into, optional)] id: Signal<String>,
    #[prop(into, optional)] value: Option<Signal<TimeDelta>>,
    #[prop(optional)] default_value: Option<TimeDelta>,
    #[prop(into, optional)] on_change: OnChange,

    #[prop(optional)] delta_hour: Option<DeltaHour>,
    #[prop(optional)] delta_minute: Option<DeltaMinute>,
    #[prop(optional)] delta_second: Option<DeltaSecond>,
    #[prop(optional)] delta_milli: Option<DeltaMilli>,

    #[prop(default=false.into(), into)] use_single_form_value: Signal<bool>,
) -> impl IntoView {
    let (default_value, set_default_value) = signal(default_value.unwrap_or_default());

    let value = Memo::new(move |_| {
        if let Some(v) = value {
            v.get()
        } else {
            default_value.get()
        }
    });

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

    let pad_input = move |node_ref: NodeRef<html::Input>, w| {
        // we check whether a signal is disposed so we know the node_ref is disposed as well
        if value.try_get().is_none() {
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

    let pad_hours = move || format!("{:02}", value().num_hours());
    let pad_mins = move || format!("{:02}", value().num_minutes() % 60);
    let pad_secs = move || format!("{:02}", value().num_seconds() % 60);
    let pad_millis = move || format!("{:03}", value().num_milliseconds() % 1000);

    let handle_change_hour = move |ev| {
        let ev_value = event_target_value(&ev);
        if let Ok(ev) = event_target_value(&ev).parse::<i64>() {
            let diff = ev - value.get().num_hours();
            set_default_value(value.get() + TimeDelta::hours(diff))
        }
    };

    let create_name = move |part| {
        if !use_single_form_value.get() {
            format!("{}[{}]", name.get(), part)
        } else {
            String::default()
        }
    };

    view! {
        <Show when=move || label.is_some()>
            <label for=id style:grid-column="1">
                {label.unwrap()()}
            </label>
        </Show>
        <div class=style::input style:grid-column="2">
            <Show when=use_single_form_value>
                <input
                    type="hidden"
                    prop:value=move || value.get().num_milliseconds()
                    name=name
                />
            </Show>
            <input
                node_ref=hour_ref
                type="number"
                prop:value=pad_hours
                style:width="4ch"
                style:text-align="end"
                on:change=handle_change_hour
                on:focusout=move |_| pad_input(hour_ref, 2)
                {..delta_hour.clone().map(|d| (d.attrs)()).unwrap_or(().into_any_attr())}
                name=move || create_name("hour")
                id=id
            />
            :
            <input
                node_ref=min_ref
                type="number"
                max="59"
                prop:value=pad_mins
                style:width="2ch"
                style:text-align="end"
                on:input=move |ev| limit_num(ev, min_ref, 0, 59)
                on:focusout=move |_| pad_input(min_ref, 2)
                {..delta_minute.clone().map(|d| (d.attrs)()).unwrap_or(().into_any_attr())}
                id="mins"
                name=move || create_name("mins")
            />
            :
            <input
                node_ref=sec_ref
                type="number"
                max="59"
                prop:value=pad_secs
                style:width="2ch"
                style:text-align="end"
                on:input=move |ev| limit_num(ev, sec_ref, 0, 59)
                on:focusout=move |_| pad_input(sec_ref, 2)
                {..delta_second.clone().map(|d| (d.attrs)()).unwrap_or(().into_any_attr())}
                id="secs"
                name=move || create_name("secs")
            />
            .
            <input
                node_ref=millis_ref
                type="number"
                max="999"
                prop:value=pad_millis
                style:width="3ch"
                style:text-align="end"
                on:input=move |ev| limit_num(ev, millis_ref, 0, 999)
                on:focusout=move |_| pad_input(millis_ref, 3)
                {..delta_milli.clone().map(|d| (d.attrs)()).unwrap_or(().into_any_attr())}
                id="millis"
                name=move || create_name("millis")
            />
        </div>
    }
}
