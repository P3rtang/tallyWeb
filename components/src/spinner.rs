use leptos::*;

#[component]
pub fn Spinner(#[prop(default = 32)] size: usize) -> impl IntoView {
    let style = format!(
        ".loader {{
            border: {border}px solid #f3f3f3; /* Light grey */
            border-top: {border}px solid #555555; /* Blue */
            border-radius: 50%;
            width: {size}px;
            height: {size}px;
            animation: spin 2s linear infinite;
        }}

        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}",
        border = size / 8
    );
    view! {
        <style>{style}</style>
        <div class="loader"></div>
    }
}
