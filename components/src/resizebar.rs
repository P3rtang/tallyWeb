use super::*;
use leptos::*;

#[derive(Debug, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
}

#[component]
pub fn ResizeBar(direction: Direction, #[prop(into)] position: Prop<usize>) -> impl IntoView {
    let position = StoredValue::new(position);

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

    let top = move || match direction {
        Direction::Vertical => "".to_string(),
        Direction::Horizontal => format!("{}px", position.get_value()() - 6),
    };

    let left = move || match direction {
        Direction::Vertical => format!("{}px", position.get_value()() - 6),
        Direction::Horizontal => "".to_string(),
    };

    view! {
        <resize-bar
            style:cursor=cursor
            style:min-width=width
            style:min-height=height
            style:position="fixed"
            style:top=top
            style:left=left
            draggable="true"
        >
            <div style:min-height="100%" style:min-width="100%"></div>
        </resize-bar>
    }
}
