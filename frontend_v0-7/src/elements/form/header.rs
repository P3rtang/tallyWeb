use super::*;

stylance::import_style!(style, "./form.module.scss");

#[slot]
pub struct HeaderSlot {
    #[prop(into)]
    title: Signal<String>,

    #[prop(into, optional_no_strip)]
    close_href: Option<Signal<String>>,

    #[prop(optional)]
    children: Option<ChildrenFn>,
}

impl Default for HeaderSlot {
    fn default() -> Self {
        Self {
            title: "Form".to_string().into(),
            close_href: None,
            children: None,
        }
    }
}

impl IntoRender for HeaderSlot {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let children = move || {
            if let Some(children) = self.children {
                children()
            } else {
                ().into_any()
            }
        };

        view! {
            <div class=style::header>
                <span>{ self.title }</span>
                <div style:display="flex" class=style::actions>
                    {children()}
                    <Show when=move || self.close_href.is_some()>
                        <button class="hover-darken icon">
                            <a href=self.close_href>
                                <img height="28px" width="28px" src="/icons/tallyweb-cross-white.svg" />
                            </a>
                        </button>
                    </Show>
                </div>
            </div>
        }
        .into_any()
    }
}
