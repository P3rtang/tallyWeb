use leptos::*;

#[derive(Debug, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
}

#[component]
pub fn ResizeBar(
    direction: Direction,
    #[prop(into)] position: MaybeSignal<usize>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let cursor = match direction {
        Direction::Vertical => "col-resize",
        Direction::Horizontal => "row-resize",
    };

    let width = match direction {
        Direction::Vertical => "12px",
        Direction::Horizontal => "100%",
    };

    let height = match direction {
        Direction::Vertical => "100%",
        Direction::Horizontal => "12px",
    };

    let pos = move || match direction {
        Direction::Vertical => (None, Some(format!("{}px", position() - 6))),
        Direction::Horizontal => (Some(format!("{}px", position() - 6)), None),
    };

    view! {
        <resize-bar
            style:cursor=cursor
            style:min-width=width
            style:min-height=height
            style:position="absolute"
            style:top=move || pos().0
            style:left=move || pos().1
            style:z-index="100"
            draggable="true"
            {..attrs}
        >
            <div style:min-height="100%" style:min-width="100%"></div>
        </resize-bar>
    }
}
