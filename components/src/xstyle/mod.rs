use super::*;

// modules
mod border;
mod padding;
mod size;

// imports
// internal
// re-exports
pub use border::XBorderRadius;
pub use padding::XPadding;
pub use size::XSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssStyleKind {
    Height,
    MinHeight,
    MaxHeight,
    Width,
    MinWidth,
    MaxWidth,
    Padding,
    BorderRadius,
}

use CssStyleKind::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssStyle {
    Height(XSize),
    MinHeight(XSize),
    MaxHeight(XSize),
    Width(XSize),
    MinWidth(XSize),
    MaxWidth(XSize),
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
                CssStyle::Height(xsize) => xsize.to_string(),
                CssStyle::MinHeight(xsize) => xsize.to_string(),
                CssStyle::MaxHeight(xsize) => xsize.to_string(),
                CssStyle::Width(xsize) => xsize.to_string(),
                CssStyle::MinWidth(xsize) => xsize.to_string(),
                CssStyle::MaxWidth(xsize) => xsize.to_string(),
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
            view! { <{..} style:border-radius=br.to_string() /> }.into_any_attr()
        } else {
            ().into_any_attr()
        }
    }
}

impl IntoAnyAttribute for XStyle {
    fn into_any_attr(self) -> AnyAttribute {
        let get_style = |style: CssStyleKind| {
            self.0
                .get_value()
                .get(&style)
                .map(CssStyle::to_string)
                .unwrap_or_default()
        };

        view! {
            <{..}
                style:padding=get_style(Padding)
                style:border-radius=get_style(BorderRadius)
                style:height=get_style(Height)
                style:min-height=get_style(MinHeight)
                style:max-height=get_style(MaxHeight)
                style:width=get_style(Width)
                style:min-width=get_style(MinWidth)
                style:max-width=get_style(MaxWidth)
            />
        }
        .into_any_attr()
    }
}

#[macro_export]
macro_rules! xstyle {
    () => { XStyle(StoredValue::new(std::collections::HashMap::new())) };

    ($type:tt: $val:expr) => {
        xstyle!($type: $val,)
    };

    (
        "width": $w:expr,
        $($type:tt: $val:expr),*
        $(,)?
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::Width, CssStyle::Width($w.into()));
        });
        map
    }};

    (
        "min-width": $w:expr,
        $($type:tt: $val:expr),*
        $(,)?
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::MinWidth, CssStyle::MinWidth($w.into()));
        });
        map
    }};

    (
        "max-width": $w:expr,
        $($type:tt: $val:expr),*
        $(,)?
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::MaxWidth, CssStyle::MaxWidth($w.into()));
        });
        map
    }};

    (
        "height": $h:expr,
        $($type:tt: $val:expr),*
        $(,)?
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::Height, CssStyle::Height($h.into()));
        });
        map
    }};

    (
        "min-height": $h:expr,
        $($type:tt: $val:expr),*
        $(,)?
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::MinHeight, CssStyle::MinHeight($h.into()));
        });
        map
    }};

    (
        "max-height": $h:expr,
        $($type:tt: $val:expr),*
        $(,)?
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::MaxHeight, CssStyle::MaxHeight($h.into()));
        });
        map
    }};

    (
        "padding": $pad:expr,
        $($type:tt: $val:expr),*
        $(,)?
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::Padding, CssStyle::Padding($pad.into()));
        });
        map
    }};

    (
        "border-radius": $br:expr,
        $($type:tt: $val:expr),*
        $(,)?
    ) => {{
        let mut map = xstyle!($($type: $val),*);
        map.update(|m| {
            m.insert(CssStyleKind::BorderRadius, CssStyle::BorderRadius($br.into()));
        });
        map
    }}
}
