use leptos::*;

#[component]
pub fn Slider<OC>(
    #[prop(into)] value: MaybeSignal<bool>,
    #[prop(default=String::new(), into)] name: String,
    on_checked: OC,
    #[prop(optional, into)] accent_color: Option<Signal<String>>,
) -> impl IntoView
where
    OC: Fn(ev::Event) + 'static,
{
    let value = create_rw_signal(value.get_untracked());

    let style = move || {
        if value() {
            format!(
                "background-color: {}",
                accent_color
                    .map(|ac| ac())
                    .unwrap_or(String::from("#8BE9FD"))
            )
        } else {
            String::new()
        }
    };

    let on_toggle = move |ev| {
        on_checked(ev);
        value.update(|v| *v = !*v);
    };

    view! {
        <label class="switch">
            <input
                type="checkbox"
                name=name
                id="show_separator"
                class="edit"
                checked=value
                on:change=on_toggle
            />
            <span class="slider" style=style></span>
        </label>
    }
}
