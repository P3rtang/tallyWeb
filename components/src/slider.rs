use leptos::*;

#[component]
pub fn slider(
    #[prop(into)] checked: (Signal<bool>, SignalSetter<bool>),
    #[prop(optional, into)] accent_color: Option<Signal<String>>,
) -> impl IntoView {
    let style = move || {
        if checked.0() {
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

    let toggle_checked = move |_| checked.1.set(!checked.0());

    view! {
        <label class="switch">
            <input
                type="checkbox"
                name="show_separator"
                id="show_separator"
                class="edit"
                prop:checked=checked.0
                on:change=toggle_checked
            />
            <span class="slider" style=style></span>
        </label>
    }
}
