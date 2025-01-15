use super::*;

use leptos_router::params::{IntoParam, ParamsError};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct CountableId(pub uuid::Uuid);

unsafe impl Send for CountableId {}

impl IntoParam for CountableId {
    fn into_param(value: Option<&str>, name: &str) -> Result<Self, ParamsError> {
        Ok(
            uuid::Uuid::parse_str(value.ok_or(ParamsError::MissingParam(name.into()))?)
                .map_err(|err| ParamsError::Params(std::sync::Arc::new(err)))?
                .into(),
        )
    }
}

impl From<uuid::Uuid> for CountableId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<CountableId> for uuid::Uuid {
    fn from(val: CountableId) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Countable {
    Counter(Arc<Mutex<Counter>>),
    Phase(Arc<Mutex<Phase>>),
    Chain(Arc<Mutex<Chain>>),
}

impl From<Countable> for CountableId {
    fn from(value: Countable) -> Self {
        value.uuid().into()
    }
}

impl From<&Countable> for CountableId {
    fn from(value: &Countable) -> Self {
        value.uuid().into()
    }
}

impl Countable {
    pub fn new(
        name: &str,
        kind: CountableKind,
        owner_uuid: uuid::Uuid,
        parent: Option<CountableId>,
    ) -> Self {
        match kind {
            CountableKind::Counter => Self::Counter(Arc::new(Mutex::new(Counter::new(
                name.into(),
                owner_uuid,
                parent,
            )))),
            CountableKind::Phase => Self::Phase(Arc::new(Mutex::new(Phase::new(
                name.into(),
                owner_uuid,
                parent.expect("Phase has to have a parent"),
            )))),
            CountableKind::Chain => todo!(),
        }
    }

    pub fn add_child_checked(&self, child: CountableId) -> AppResult<()> {
        match self {
            Countable::Counter(c) => {
                c.lock()?.children.push(child);
                Ok(())
            }
            Countable::Phase(_) => Err(AppError::CannotContainChildren("Phase".into())),
            Countable::Chain(_) => Err(AppError::CannotContainChildren("Chain".into())),
        }
    }

    pub fn add_child(&self, child: CountableId) {
        self.add_child_checked(child).unwrap()
    }

    pub fn uuid_checked(&self) -> AppResult<uuid::Uuid> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.uuid,
            Countable::Phase(p) => p.lock()?.uuid,
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid_checked().unwrap()
    }

    pub fn name_checked(&self) -> AppResult<String> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.name.clone(),
            Countable::Phase(p) => p.lock()?.name.clone(),
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn name(&self) -> String {
        self.name_checked().unwrap()
    }

    pub fn created_at_checked(&self) -> AppResult<chrono::NaiveDateTime> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.created_at,
            Countable::Phase(p) => p.lock()?.created_at,
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at_checked().unwrap()
    }

    pub fn last_edit_checked(&self) -> AppResult<chrono::NaiveDateTime> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.last_edit,
            Countable::Phase(p) => p.lock()?.last_edit,
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn last_edit(&self) -> chrono::NaiveDateTime {
        self.last_edit_checked().unwrap()
    }

    pub fn is_archived_checked(&self) -> AppResult<bool> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.is_deleted,
            Countable::Phase(p) => p.lock()?.is_deleted,
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn is_archived(&self) -> bool {
        self.is_archived_checked().unwrap()
    }

    pub fn as_js(&self) -> AppResult<wasm_bindgen::JsValue> {
        Ok(js_sys::JSON::parse(&serde_json::to_string(&self)?)?)
    }

    pub fn from_js(val: wasm_bindgen::JsValue) -> AppResult<Self> {
        let this = serde_json::from_str(
            &js_sys::JSON::stringify(&val)?
                .as_string()
                .unwrap_or_default(),
        )?;
        Ok(this)
    }

    fn set_edit(&self) -> AppResult<()> {
        match self {
            Countable::Counter(c) => c.lock()?.last_edit = chrono::Utc::now().naive_utc(),
            Countable::Phase(p) => p.lock()?.last_edit = chrono::Utc::now().naive_utc(),
            Countable::Chain(_) => todo!(),
        }

        Ok(())
    }

    fn set_changed(&self, set: bool) -> AppResult<()> {
        match self {
            Countable::Counter(c) => c.lock()?.has_change = set,
            Countable::Phase(p) => p.lock()?.has_change = set,
            Countable::Chain(_) => todo!(),
        }

        Ok(())
    }
}

impl Savable for Vec<Countable> {
    fn has_change(&self) -> bool {
        self.iter().any(|c| c.has_change())
    }
}

impl ServerSavable for Vec<Countable> {
    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), leptos::prelude::ServerFnError>>
                + Send
                + Sync,
        >,
    > {
        _ = self.iter().map(|c| c.set_changed(false));
        Box::pin(api::update_countable_many(self.clone()))
    }
}

impl LocalSavable for Vec<Countable> {
    fn indexed_db_name(&self) -> String {
        "Countable".into()
    }

    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + 'a>> {
        use wasm_bindgen::JsValue;

        Box::pin(async move {
            for c in self {
                let key = JsValue::from_str(&c.uuid().to_string());
                let value = c.as_js();

                if let Some(old_val) = obj
                    .get(&key)
                    .await?
                    .and_then(|v| Countable::from_js(v).ok())
                {
                    if old_val.last_edit() > c.last_edit() {
                        continue;
                    }
                }

                let _ = self.iter().try_for_each(|c| c.set_edit());
                obj.put_kv(&key, &value?).await?;
            }

            Ok(())
        })
    }
}

impl Savable for Countable {
    fn has_change(&self) -> bool {
        match self {
            Countable::Counter(c) => c.lock().map(|c| c.has_change).unwrap_or(false),
            Countable::Phase(p) => p.lock().map(|p| p.has_change).unwrap_or(false),
            Countable::Chain(_) => false,
        }
    }
}

impl ServerSavable for Countable {
    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = leptos::prelude::Result<(), leptos::prelude::ServerFnError>,
                > + Send
                + Sync,
        >,
    > {
        vec![self.clone()].save_endpoint()
    }
}

impl LocalSavable for Countable {
    fn indexed_db_name(&self) -> String {
        "Countable".into()
    }

    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<()>> + 'a>> {
        use wasm_bindgen::JsValue;
        let key = JsValue::from_str(&self.uuid().to_string());
        let value = self.as_js();
        Box::pin(async move {
            obj.put_kv(&key, &value?).await?;
            Ok(())
        })
    }
}

impl PartialEq for Countable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Countable::Counter(a), Countable::Counter(b)) => match (a.try_lock(), b.try_lock()) {
                (Ok(a), Ok(b)) => a.eq(&*b),
                _ => false,
            },
            (Countable::Phase(a), Countable::Phase(b)) => match (a.try_lock(), b.try_lock()) {
                (Ok(a), Ok(b)) => a.eq(&*b),
                _ => false,
            },
            (Countable::Chain(a), Countable::Chain(b)) => match (a.try_lock(), b.try_lock()) {
                (Ok(a), Ok(b)) => a.eq(&*b),
                _ => false,
            },
            _ => false,
        }
    }
}

impl Eq for Countable {}

#[cfg(feature = "ssr")]
impl From<backend::DbCounter> for Countable {
    fn from(value: backend::DbCounter) -> Self {
        Self::Counter(Arc::new(Mutex::new(Counter {
            uuid: value.uuid,
            owner_uuid: value.owner_uuid,
            parent: None,
            children: Vec::new(),
            name: value.name,
            last_edit: value.last_edit,
            created_at: value.created_at,
            is_deleted: value.is_deleted,
            has_change: false,
        })))
    }
}

#[cfg(feature = "ssr")]
impl From<backend::DbPhase> for Countable {
    fn from(value: backend::DbPhase) -> Self {
        Self::Phase(Arc::new(Mutex::new(Phase {
            uuid: value.uuid,
            owner_uuid: value.owner_uuid,
            parent: value.parent_uuid.into(),
            name: value.name,
            count: value.count,
            time: chrono::Duration::milliseconds(value.time),
            hunt_type: value.hunt_type.into(),
            has_charm: value.has_charm,
            success: value.success,
            last_edit: value.last_edit,
            created_at: value.created_at,
            is_deleted: value.is_deleted,
            step_size: value.step_size,
            has_change: false,
        })))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum CountableKind {
    #[default]
    Counter,
    Phase,
    Chain,
}

impl std::fmt::Display for CountableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Counter => write!(f, "Counter"),
            Self::Phase => write!(f, "Phase"),
            Self::Chain => write!(f, "Chain"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counter {
    uuid: uuid::Uuid,
    owner_uuid: uuid::Uuid,
    parent: Option<CountableId>,
    #[serde(default)]
    children: Vec<CountableId>,
    name: String,
    last_edit: chrono::NaiveDateTime,
    created_at: chrono::NaiveDateTime,
    is_deleted: bool,
    has_change: bool,
}

impl Counter {
    fn new(name: String, owner_uuid: uuid::Uuid, parent: Option<CountableId>) -> Self {
        Self {
            uuid: uuid::Uuid::new_v4(),
            owner_uuid,
            parent,
            children: Vec::new(),
            name,
            last_edit: chrono::Utc::now().naive_utc(),
            created_at: chrono::Utc::now().naive_utc(),
            is_deleted: false,
            has_change: false,
        }
    }

    pub fn owner_uuid(&self) -> uuid::Uuid {
        self.owner_uuid
    }

    pub fn parent(&self) -> Option<CountableId> {
        self.parent
    }

    pub fn set_parent(&mut self, parent: Option<CountableId>) {
        self.has_change = true;
        self.parent = parent;
    }

    pub fn children(&self) -> Vec<CountableId> {
        self.children.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl ToString) {
        self.has_change = true;
        self.name = name.to_string()
    }

    pub fn last_edit(&self) -> chrono::NaiveDateTime {
        self.last_edit
    }

    pub fn set_last_edit(&mut self, last_edit: chrono::NaiveDateTime) {
        self.has_change = true;
        self.last_edit = last_edit;
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    pub fn set_is_deleted(&mut self, is_deleted: bool) {
        self.has_change = true;
        self.is_deleted = is_deleted
    }
}

impl IntoIterator for Counter {
    type Item = CountableId;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

#[cfg(feature = "ssr")]
impl Into<backend::DbCounter> for Counter {
    fn into(self) -> backend::DbCounter {
        backend::DbCounter {
            uuid: self.uuid,
            owner_uuid: self.owner_uuid,
            name: self.name,
            last_edit: self.last_edit,
            created_at: self.created_at,
            is_deleted: self.is_deleted,
        }
    }
}

// #[serde_with::serde_as]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase {
    uuid: uuid::Uuid,
    owner_uuid: uuid::Uuid,
    parent: CountableId,
    name: String,
    count: i32,
    // #[serde_as(as = "serde_with::DurationMilliSeconds<i64>")]
    time: chrono::Duration,
    hunt_type: Hunttype,
    has_charm: bool,
    success: bool,
    last_edit: chrono::NaiveDateTime,
    created_at: chrono::NaiveDateTime,
    is_deleted: bool,
    step_size: i32,
    has_change: bool,
}

impl Phase {
    fn new(name: String, owner_uuid: uuid::Uuid, parent: CountableId) -> Self {
        Self {
            uuid: uuid::Uuid::new_v4(),
            owner_uuid,
            parent,
            name,
            last_edit: chrono::Utc::now().naive_utc(),
            created_at: chrono::Utc::now().naive_utc(),
            ..Default::default()
        }
    }

    pub fn owner_uuid(&self) -> uuid::Uuid {
        self.owner_uuid
    }

    pub fn parent(&self) -> CountableId {
        self.parent
    }

    pub fn set_parent(&mut self, parent: CountableId) {
        self.has_change = true;
        self.parent = parent;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl ToString) {
        self.has_change = true;
        self.name = name.to_string()
    }

    pub fn count(&self) -> i32 {
        self.count
    }

    pub fn set_count(&mut self, count: i32) {
        self.has_change = true;
        self.count = count;
    }

    pub fn add_count(&mut self, count: i32) {
        self.has_change = true;
        self.count += count;
    }

    pub fn step_count(&mut self) {
        self.has_change = true;
        self.count += self.step_size
    }

    pub fn time(&self) -> chrono::TimeDelta {
        self.time
    }

    pub fn set_time(&mut self, time: chrono::Duration) {
        self.has_change = true;
        self.time = time;
    }

    pub fn add_time(&mut self, time: chrono::Duration) {
        self.has_change = true;
        self.time += time;
    }

    pub fn hunt_type(&self) -> Hunttype {
        self.hunt_type
    }

    pub fn set_hunt_type(&mut self, hunt_type: Hunttype) {
        self.has_change = true;
        self.hunt_type = hunt_type;
    }

    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    pub fn set_is_deleted(&mut self, is_deleted: bool) {
        self.has_change = true;
        self.is_deleted = is_deleted
    }

    pub fn has_charm(&self) -> bool {
        self.has_charm
    }

    pub fn set_has_charm(&mut self, has_charm: bool) {
        self.has_change = true;
        self.has_charm = has_charm
    }

    pub fn success(&self) -> bool {
        self.success
    }

    pub fn set_success(&mut self, success: bool) {
        self.has_change = true;
        self.success = success;
    }

    pub fn toggle_success(&mut self) {
        self.has_change = true;
        self.success = !self.success;
    }

    pub fn step_size(&self) -> i32 {
        self.step_size
    }

    pub fn set_step_size(&mut self, step_size: i32) {
        self.has_change = true;
        self.step_size = step_size;
    }
}

#[cfg(feature = "ssr")]
impl Into<backend::DbPhase> for Phase {
    fn into(self) -> backend::DbPhase {
        backend::DbPhase {
            uuid: self.uuid,
            owner_uuid: self.owner_uuid,
            parent_uuid: self.parent.0,
            name: self.name,
            count: self.count,
            time: self.time.num_milliseconds(),
            hunt_type: self.hunt_type.into(),
            has_charm: self.has_charm,
            dexnav_encounters: None,
            success: self.success,
            last_edit: self.last_edit,
            created_at: self.created_at,
            is_deleted: self.is_deleted,
            step_size: self.step_size,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chain {}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Hunttype {
    #[default]
    OldOdds,
    NewOdds,
    SOS,
    // DexNav(DexNav),
    Masuda(Masuda),
    Mixed,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Masuda {
    GenIV,
    GenV,
    #[default]
    GenVI,
    // TODO: make this a generation enum
    // TODO: gen VIII+ has 6 rerolls
}

impl Hunttype {
    pub(crate) fn rolls(&self) -> impl Fn(i32, bool) -> i32 {
        match self {
            Hunttype::OldOdds => |count, has_charm: bool| (count * if has_charm { 3 } else { 1 }),
            Hunttype::NewOdds => |count, has_charm: bool| (count * if has_charm { 3 } else { 1 }),
            Hunttype::SOS => |count, has_charm: bool| match count {
                c if c < 10 => count * if has_charm { 3 } else { 1 },
                c if c < 20 => 10 + (count - 10) * if has_charm { 3 + 4 } else { 1 + 4 },
                c if c < 30 => 60 + (count - 20) * if has_charm { 3 + 8 } else { 1 + 8 },
                _ => 50 + (count - 30) * if has_charm { 3 + 13 } else { 1 + 12 },
            },
            Hunttype::Masuda(Masuda::GenIV) => {
                |count, has_charm: bool| (count * if has_charm { 3 + 4 } else { 1 + 4 })
            }
            Hunttype::Masuda(_) => {
                |count, has_charm: bool| (count * if has_charm { 3 + 5 } else { 1 + 5 })
            }
            Hunttype::Mixed => unreachable!(),
        }
    }

    pub(crate) fn odds(&self) -> f64 {
        match self {
            Hunttype::OldOdds | Hunttype::Masuda(Masuda::GenIV) => 8192.0,
            _ => 4096.0,
        }
    }

    pub fn repr(&self) -> &'static str {
        match self {
            Self::OldOdds => "Old Odds",
            Self::NewOdds => "New Odds",
            Self::SOS => "SOS",
            Self::Masuda(Masuda::GenIV) => "Masuda (gen IV)",
            Self::Masuda(Masuda::GenV) => "Masuda (gen V)",
            Self::Masuda(Masuda::GenVI) => "Masuda (gen VI+)",
            Self::Mixed => "Mixed",
        }
    }

    pub fn as_str(&self) -> &'static str {
        (*self).into()
    }
}

impl std::fmt::Display for Hunttype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &'static str = (*self).into();
        write!(f, "{}", s)
    }
}

impl From<Hunttype> for &'static str {
    fn from(val: Hunttype) -> Self {
        match val {
            Hunttype::OldOdds => "OldOdds",
            Hunttype::NewOdds => "NewOdds",
            Hunttype::SOS => "SOS",
            Hunttype::Masuda(Masuda::GenIV) => "MasudaGenIV",
            Hunttype::Masuda(Masuda::GenV) => "MasudaGenV",
            Hunttype::Masuda(Masuda::GenVI) => "MasudaGenVI",
            Hunttype::Mixed => "Mixed",
        }
    }
}

impl TryFrom<String> for Hunttype {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "OldOdds" => Ok(Self::OldOdds),
            "NewOdds" => Ok(Self::NewOdds),
            "SOS" => Ok(Self::SOS),
            "MasudaGenIV" => Ok(Self::Masuda(Masuda::GenIV)),
            "MasudaGenV" => Ok(Self::Masuda(Masuda::GenV)),
            "MasudaGenVI" => Ok(Self::Masuda(Masuda::GenVI)),
            _ => Err(String::from(
                "Hunttype should be one of the following: OldOdds, NewOdds, SOS, Masuda",
            )),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<backend::Hunttype> for Hunttype {
    fn from(value: backend::Hunttype) -> Self {
        match value {
            backend::Hunttype::OldOdds => Self::OldOdds,
            backend::Hunttype::NewOdds => Self::NewOdds,
            backend::Hunttype::SOS => Self::SOS,
            backend::Hunttype::DexNav => todo!(),
            backend::Hunttype::MasudaGenIV => Self::Masuda(Masuda::GenIV),
            backend::Hunttype::MasudaGenV => Self::Masuda(Masuda::GenV),
            backend::Hunttype::MasudaGenVI => Self::Masuda(Masuda::GenVI),
        }
    }
}

#[cfg(feature = "ssr")]
impl Into<backend::Hunttype> for Hunttype {
    fn into(self) -> backend::Hunttype {
        match self {
            Self::OldOdds => backend::Hunttype::OldOdds,
            Self::NewOdds => backend::Hunttype::NewOdds,
            Self::SOS => backend::Hunttype::SOS,
            Self::Masuda(Masuda::GenIV) => backend::Hunttype::MasudaGenIV,
            Self::Masuda(Masuda::GenV) => backend::Hunttype::MasudaGenV,
            Self::Masuda(Masuda::GenVI) => backend::Hunttype::MasudaGenVI,
            Self::Mixed => unreachable!(),
        }
    }
}

impl std::ops::BitOr<Hunttype> for Hunttype {
    type Output = Hunttype;

    fn bitor(self, rhs: Hunttype) -> Self::Output {
        if self != rhs {
            Hunttype::Mixed
        } else {
            self
        }
    }
}
