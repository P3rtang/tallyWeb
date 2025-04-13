use super::*;

stylance::import_style!(style, "./icon.module.scss");

pub fn letter_to_three_digit_hash(letter: char) -> String {
    use rand::{Rng, SeedableRng};

    let mut rng = rand::rngs::StdRng::seed_from_u64(letter as u64);
    let random_hash = rng.gen_range(0x6..=0xF);
    let random_hash2 = rng.gen_range(0x6..=0xF);
    let random_hash3 = rng.gen_range(0x6..=0xF);
    format!(
        "{:x}{:x}{:x}{:x}{:x}{:x}",
        random_hash, random_hash, random_hash2, random_hash2, random_hash3, random_hash3
    )
}

#[component]
pub fn AccountIcon<F>(username: F) -> impl IntoView
where
    F: Fn() -> String + Clone + Sync + Send + 'static,
{
    let history = use_history();
    let initial = StoredValue::new(move || {
        username()
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default()
    });

    let attrs = move || view! { <{..} class=style::icon data-testid="test-account-icon" aria_label="account overlay" /> };

    let handle_pref_click = move |_| {
        history.save_location();
    };

    hoc::with_accent(move || {
        view! {
            <Menu>
                <MenuButton attrs slot>
                    <b>{move || { initial.get_value()() }}</b>
                </MenuButton>
                <MenuEntry
                    on:click=handle_pref_click
                    href="/preferences?topic=styling"
                    attr:aria_label="settings"
                >
                    <MenuEntrySlot icon=IconKind::Settings label="Preferences" slot />
                </MenuEntry>
                <MenuBreak />
                <MenuEntry attr:rel="external" href="/login">
                    <MenuEntrySlot icon=IconKind::LogOut label="Log out" slot />
                </MenuEntry>
            </Menu>
        }
    })
}
