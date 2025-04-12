use super::*;

// modules
mod border;
mod padding;

// imports
// internal
// re-exports
pub use border::XBorderRadius;
pub use padding::XPadding;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssStyleKind {
    Padding,
    BorderRadius,
}

use CssStyleKind::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssStyle {
    Padding(XPadding),
    BorderRadius(XBorderRadius),
}

impl std::fmt::Display for CssStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CssStyle::Padding(padding) => padding.to_string(),
                CssStyle::BorderRadius(br) => br.to_string(),
            }
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct XStyle(pub StoredValue<HashMap<CssStyleKind, CssStyle>>);

impl XStyle {
    #[allow(dead_code)]
    pub fn update(self, cb: impl Fn(&mut HashMap<CssStyleKind, CssStyle>)) {
        self.0.update_value(cb)
    }

    pub fn border_radius(self) -> AnyAttribute {
        if let Some(br) = self.0.get_value().get(&BorderRadius) {
            view! { <{} style:border-radius=br.to_string() /> }.into_any_attr()
        } else {
            ().into_any_attr()
        }
    }
}

impl IntoAttribute for XStyle {
    type Output = AnyAttribute;

    fn into_attr(self) -> Self::Output {
        let get_style = |style: CssStyleKind| {
            self.0
                .get_value()
                .get(&style)
                .map(CssStyle::to_string)
                .unwrap_or_default()
        };

        view! { <{} style:padding=get_style(Padding) style:border-radius=get_style(BorderRadius) /> }
        .into_any_attr()
    }
}

#[macro_export]
macro_rules! xstyle {
    () => { XStyle(StoredValue::new(std::collections::HashMap::new())) };

    (
        "padding": $pad:expr
        $(,$type:tt: $val:expr),*
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::Padding, CssStyle::Padding($pad));
        });
        map
    }};

    (
        "border-radius": $br:expr
        $(,[$type:tt: $val:expr]),*
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::BorderRadius, CssStyle::BorderRadius($br));
        });
        map
    }}
}
