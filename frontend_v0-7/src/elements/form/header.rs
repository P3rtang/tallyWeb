use super::*;

stylance::import_style!(style, "./form.module.scss");

#[slot]
pub struct HeaderSlot {
    #[prop(into)]
    title: Signal<String>,

    #[prop(into, optional_no_strip)]
    close_href: Option<Signal<String>>,

    #[prop(into, optional)]
    on_close: EventCallback<ev::MouseEvent>,

    #[prop(optional)]
    children: Option<ChildrenFn>,
}

impl Default for HeaderSlot {
    fn default() -> Self {
        Self {
            title: "Form".to_string().into(),
            on_close: Default::default(),
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

        let on_close = move |ev| self.on_close.call(ev);

        view! {
            <div class=style::header>
                <span>{ self.title }</span>
                <div style:display="flex" class=style::actions>
                    {children()}
                    <Show when=move || self.close_href.is_some()>
                        <button class="hover-darken icon">
                            <a href=self.close_href on:click=on_close.clone()>
                                <Icon kind=IconKind::Cross />
                            </a>
                        </button>
                    </Show>
                </div>
            </div>
        }
        .into_any()
    }
}
