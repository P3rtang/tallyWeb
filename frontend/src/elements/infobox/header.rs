use super::*;

#[derive(Debug, Clone, Copy)]
enum VisibleField {
    Count,
    Time,
    Progress,
    LastStep,
    AvgStep,
}

impl std::fmt::Display for VisibleField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VisibleField::Count => "Count",
                VisibleField::Time => "Time",
                VisibleField::Progress => "Progress",
                VisibleField::LastStep => "LastStep",
                VisibleField::AvgStep => "AvgStep",
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VisibleInfo {
    count: Option<bool>,
    time: Option<bool>,
    progress: Option<bool>,
    last_step: Option<bool>,
    avg_step: Option<bool>,

    on_mobile: Memo<bool>,
}

impl Default for VisibleInfo {
    fn default() -> Self {
        let on_mobile = use_breakpoint(ViewPort::Small, true);

        Self {
            count: None,
            time: None,
            progress: None,
            last_step: None,
            avg_step: None,
            on_mobile,
        }
    }
}

impl VisibleInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self) -> bool {
        self.count.unwrap_or(true)
    }

    pub fn toggle_count(&mut self) {
        self.count = Some(!self.count())
    }

    pub fn time(&self) -> bool {
        self.time.unwrap_or(true)
    }

    pub fn toggle_time(&mut self) {
        self.time = Some(!self.time())
    }

    pub fn progress(&self) -> bool {
        self.progress.unwrap_or(!self.on_mobile.get())
    }

    pub fn toggle_progress(&mut self) {
        self.progress = Some(!self.progress())
    }

    pub fn last_step(&self) -> bool {
        self.last_step.unwrap_or(!self.on_mobile.get())
    }

    pub fn toggle_last_step(&mut self) {
        self.last_step = Some(!self.last_step())
    }

    pub fn avg_step(&self) -> bool {
        self.avg_step.unwrap_or(!self.on_mobile.get())
    }

    pub fn toggle_avg_step(&mut self) {
        self.avg_step = Some(!self.avg_step())
    }

    fn get(&self, field: VisibleField) -> bool {
        match field {
            VisibleField::Count => self.count(),
            VisibleField::Time => self.time(),
            VisibleField::Progress => self.progress(),
            VisibleField::LastStep => self.last_step(),
            VisibleField::AvgStep => self.avg_step(),
        }
    }

    fn icon(&self, field: VisibleField) -> IconKind {
        let is_visible = self.get(field);

        if is_visible {
            IconKind::Visible
        } else {
            IconKind::NotVisible
        }
    }

    fn toggle(&mut self, field: VisibleField) {
        match field {
            VisibleField::Count => self.toggle_count(),
            VisibleField::Time => self.toggle_time(),
            VisibleField::Progress => self.toggle_progress(),
            VisibleField::LastStep => self.toggle_last_step(),
            VisibleField::AvgStep => self.toggle_avg_step(),
        }
    }
}

#[component]
pub fn InfoHeader(
    #[prop(into)] key: Signal<CountableId>,
    visible_info: RwSignal<VisibleInfo>,
) -> impl IntoView {
    let history = use_history();
    let params = use_params::<UserName>();
    let is_small = use_breakpoint(ViewPort::Small, true);

    let session = expect_context::<RwSignal<UserSession>>();
    let store = expect_context::<RwSignal<CountableStore>>();
    let selection = expect_context::<Memo<Selection>>();

    let countable_name = Signal::derive(move || store.get().name(&key.get()));
    let user_name = move || params.get().map(|p| p.id).ok();

    let edit_link = Signal::derive(move || format!("edit?slct={}", key.get().0));

    let on_click = move |_| {
        history.save_location();
    };

    view! {
        <div class=style::header>
            <div>
                <InfoPickMenu visible_info />
                <Text style:font-size=move || {
                    if is_small.get() { "24px" } else { "28px" }
                }>{countable_name}</Text>
                <div class=style::actions>
                    <Button href=edit_link on:click=on_click class="hover-darken icon">
                        <Icon kind=IconKind::Edit />
                    </Button>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn InfoPickMenu(visible_info: RwSignal<VisibleInfo>) -> impl IntoView {
    let menu_attrs = view! { <{..} attr:class=style::menu /> }.into_attr_fn();

    view! {
        <Menu menu_attrs>
            <MenuButton slot>
                <Icon kind=IconKind::HamburgerMenu />
            </MenuButton>
            <MenuEntry>
                <InfoPickMenuEntry field=VisibleField::Count visible_info />
            </MenuEntry>
            <MenuEntry>
                <InfoPickMenuEntry field=VisibleField::Time visible_info />
            </MenuEntry>
            <MenuEntry>
                <InfoPickMenuEntry field=VisibleField::Progress visible_info />
            </MenuEntry>
            <MenuEntry>
                <InfoPickMenuEntry field=VisibleField::LastStep visible_info />
            </MenuEntry>
            <MenuEntry>
                <InfoPickMenuEntry field=VisibleField::AvgStep visible_info />
            </MenuEntry>
        </Menu>
    }
}

#[component]
fn InfoPickMenuEntry(field: VisibleField, visible_info: RwSignal<VisibleInfo>) -> impl IntoView {
    let on_click = move |ev: MouseEvent| {
        ev.stop_propagation();

        visible_info.update(|i| i.toggle(field))
    };

    let icon_kind = Signal::derive(move || visible_info.with(|v| v.icon(field)));

    view! {
        <div class=style::entry>
            <Button
                style:background="transparent"
                attr:id=field.to_string().to_lowercase()
                on:click=on_click
                xstyle=xstyle!("padding": XPadding::Medium)
            >
                <Icon kind=icon_kind />
            </Button>
            <label on:click=move |ev| ev.stop_propagation() for=field.to_string().to_lowercase()>
                {field.to_string()}
            </label>
        </div>
    }
}
