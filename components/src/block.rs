use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub enum Direction {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::Row => "row",
                Direction::Column => "column",
                Direction::RowReverse => "row-reverse",
                Direction::ColumnReverse => "column-reverse",
            }
        )
    }
}

impl Into<std::borrow::Cow<'static, str>> for Direction {
    fn into(self) -> std::borrow::Cow<'static, str> {
        self.to_string().into()
    }
}

#[component]
pub fn Block(
    children: ChildrenFn,
    #[prop(optional)] spacing: usize,
    #[prop(optional, into)] direction: Signal<Direction>,
    #[prop(optional_no_strip, into)] tooltip: Signal<Option<String>>,
) -> impl IntoView {
    with_tooltip(
        move || {
            view! {
                <div
                    style:display="flex"
                    style:flex-direction=direction
                    style:gap=format!("{}px", spacing)
                >
                    {children()}
                </div>
            }
        },
        tooltip,
    )
}
