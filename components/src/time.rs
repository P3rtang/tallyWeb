use chrono::format::*;
use leptos::*;
use std::io::Write;

const MINUTE_SECONDS: u64 = 60;
const HOUR_SECONDS: u64 = MINUTE_SECONDS * 60;
const DAY_SECONDS: u64 = HOUR_SECONDS * 24;

#[component]
pub fn Clock(
    #[prop(optional, into)] value: Option<MaybeSignal<std::time::Duration>>,
    #[prop(optional, default="%H:%M:%S".into(), into)] format: MaybeSignal<String>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let time = move || {
        if let Some(v) = value {
            let chrono_val = chrono::Duration::from_std(v()).unwrap();
            let date = chrono::DateTime::UNIX_EPOCH
                .checked_add_signed(chrono_val)
                .unwrap();
            date.format(&format()).to_string()
        } else {
            chrono::Local::now().format(&format()).to_string()
        }
    };

    view! { <span {..attrs}>{time}</span> }
}

#[component]
pub fn Timer(
    #[prop(into)] value: MaybeSignal<std::time::Duration>,
    #[prop(optional, default="%H:%M:%S".into(), into)] format: MaybeSignal<String>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let value = create_memo(move |_| value.get());
    // TODO: factor this out into function
    let stringified = Signal::derive(move || {
        let format = format();
        let strf_items = StrftimeItems::new(&format);
        let mut writer = Vec::with_capacity(32);
        for (idx, item) in strf_items.enumerate() {
            match item {
                Item::Literal(l) => write!(writer, "{}", l),
                Item::OwnedLiteral(l) => write!(writer, "{}", l),
                Item::Space(s) => write!(writer, "{}", s),
                Item::OwnedSpace(s) => write!(writer, "{}", s),
                Item::Numeric(typ, pad) => {
                    let (mut val, modulo): (i128, i32) = match typ {
                        Numeric::Year => todo!(),
                        Numeric::YearDiv100 => todo!(),
                        Numeric::YearMod100 => todo!(),
                        Numeric::IsoYear => todo!(),
                        Numeric::IsoYearDiv100 => todo!(),
                        Numeric::IsoYearMod100 => todo!(),
                        Numeric::Month => todo!(),
                        Numeric::Day => ((value().as_secs() / DAY_SECONDS).into(), 28),
                        Numeric::WeekFromSun => todo!(),
                        Numeric::WeekFromMon => todo!(),
                        Numeric::IsoWeek => todo!(),
                        Numeric::NumDaysFromSun => todo!(),
                        Numeric::WeekdayFromMon => todo!(),
                        Numeric::Ordinal => todo!(),
                        Numeric::Hour => ((value().as_secs() / 3600).into(), 24),
                        Numeric::Hour12 => todo!(),
                        Numeric::Minute => ((value().as_secs() / 60).into(), 60),
                        Numeric::Second => ((value().as_secs()).into(), 60),
                        Numeric::Nanosecond => ((value().as_nanos()) as i128, 1_000_000_000),
                        Numeric::Timestamp => todo!(),
                        Numeric::Internal(_) => todo!(),
                        _ => unreachable!(),
                    };

                    if idx != 0 && modulo > 0 {
                        val %= modulo as i128
                    };

                    match pad {
                        Pad::None => write!(writer, "{}", val),
                        Pad::Zero => write!(writer, "{:0width$}", val, width = 2),
                        Pad::Space => write!(writer, "{:width$}", val, width = 2),
                    }
                }
                Item::Fixed(f) => match f {
                    Fixed::ShortMonthName => todo!(),
                    Fixed::LongMonthName => todo!(),
                    Fixed::ShortWeekdayName => todo!(),
                    Fixed::LongWeekdayName => todo!(),
                    Fixed::LowerAmPm => todo!(),
                    Fixed::UpperAmPm => todo!(),
                    Fixed::Nanosecond => write!(writer, ".{}", value().as_nanos() % 1_000_000_000),
                    Fixed::Nanosecond3 => {
                        write!(writer, ".{:03}", value().as_nanos() / 1_000_000 % 1_000)
                    }
                    Fixed::Nanosecond6 => {
                        write!(writer, ".{:06}", value().as_nanos() / 1_000 % 1_000_000)
                    }
                    Fixed::Nanosecond9 => {
                        write!(writer, ".{:09}", value().as_nanos() % 1_000_000_000)
                    }
                    Fixed::TimezoneName => todo!(),
                    Fixed::TimezoneOffsetColon => todo!(),
                    Fixed::TimezoneOffsetDoubleColon => todo!(),
                    Fixed::TimezoneOffsetTripleColon => todo!(),
                    Fixed::TimezoneOffsetColonZ => todo!(),
                    Fixed::TimezoneOffset => todo!(),
                    Fixed::TimezoneOffsetZ => todo!(),
                    Fixed::RFC2822 => todo!(),
                    Fixed::RFC3339 => todo!(),
                    Fixed::Internal(_) => todo!(),
                    _ => unreachable!(),
                },
                Item::Error => todo!(),
            }
            // TODO: look into removing this unwrap
            .unwrap();
        }

        String::from_utf8(writer).unwrap_or_default()
    });

    view! { <span {..attrs}>{stringified}</span> }
}
