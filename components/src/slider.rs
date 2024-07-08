use leptos::*;

#[component]
pub fn Slider<OC>(
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    #[prop(into)] checked: MaybeSignal<bool>,
    on_checked: OC,
) -> impl IntoView
where
    OC: Fn(ev::Event) + 'static,
{
    let on_toggle = move |ev| {
        on_checked(ev);
    };

    view! {
        <label>
            <style>
                r#"
                switch-el {
                    position: relative;
                    display: inline-block;
                    min-width: 60px;
                    max-width: 60px;
                    height: 34px;
                    margin: auto 0px;
                }
                
                slider-el {
                    position: absolute;
                    cursor: pointer;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: var(--accent, #3584E4);
                    -webkit-transition: .32s;
                    transition: .32s;
                    border-radius: 34px;
                }
                
                slider-el:before {
                    position: absolute;
                    content: "";
                    height: 26px;
                    width: 26px;
                    left: 4px;
                    bottom: 4px;
                    background: white;
                    -webkit-transition: .32s;
                    transition: .32s;
                    border-radius: 50%;
                }
                
                input:focus+slider-el {
                    box-shadow: 0 0 1px #2196F3;
                }
                
                input:checked+slider-el:before {
                    -webkit-transform: translateX(26px);
                    -ms-transform: translateX(26px);
                    transform: translateX(26px);
                }
                
                input:not(:checked)+slider-el {
                    background: #CCC;
                }
                
                input:disabled+slider-el {
                    filter: brightness(60%);
                }
                
                @keyframes slide {
                    100% {
                        left: 0;
                    }
                }
                "#
            </style>
            <switch-el>
                <input
                    type="checkbox"
                    {..attrs}
                    style:display="none"
                    on:change=on_toggle
                    prop:checked=checked
                    checked=checked
                />
                <slider-el class="slider"></slider-el>
            </switch-el>
        </label>
    }
}
