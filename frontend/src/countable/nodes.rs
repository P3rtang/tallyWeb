use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use super::*;
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct CountableId(uuid::Uuid);

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

    pub fn add_child_checked(&self, child: CountableId) -> Result<(), AppError> {
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

    pub fn uuid_checked(&self) -> Result<uuid::Uuid, AppError> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.uuid,
            Countable::Phase(p) => p.lock()?.uuid,
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid_checked().unwrap()
    }

    pub fn name_checked(&self) -> Result<String, AppError> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.name.clone(),
            Countable::Phase(p) => p.lock()?.name.clone(),
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn name(&self) -> String {
        self.name_checked().unwrap()
    }

    pub fn set_name_checked(&self, name: &str) -> Result<(), AppError> {
        match self {
            Countable::Counter(c) => c.lock()?.name = name.into(),
            Countable::Phase(p) => p.lock()?.name = name.into(),
            Countable::Chain(_) => todo!(),
        }

        Ok(())
    }

    pub fn set_name(&self, name: &str) {
        self.set_name_checked(name).unwrap()
    }

    pub fn created_at_checked(&self) -> Result<chrono::NaiveDateTime, AppError> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.created_at,
            Countable::Phase(p) => p.lock()?.created_at,
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at_checked().unwrap()
    }

    pub fn last_edit_checked(&self) -> Result<chrono::NaiveDateTime, AppError> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.last_edit,
            Countable::Phase(p) => p.lock()?.last_edit,
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn last_edit(&self) -> chrono::NaiveDateTime {
        self.last_edit_checked().unwrap()
    }

    pub fn is_archived_checked(&self) -> Result<bool, AppError> {
        Ok(match self {
            Countable::Counter(c) => c.lock()?.is_deleted,
            Countable::Phase(p) => p.lock()?.is_deleted,
            Countable::Chain(_) => todo!(),
        })
    }

    pub fn is_archived(&self) -> bool {
        self.is_archived_checked().unwrap()
    }

    pub fn as_js(&self) -> Result<wasm_bindgen::JsValue, AppError> {
        Ok(js_sys::JSON::parse(&serde_json::to_string(&self)?)?)
    }

    pub fn from_js(val: wasm_bindgen::JsValue) -> Result<Self, AppError> {
        let this = serde_json::from_str(
            &js_sys::JSON::stringify(&val)?
                .as_string()
                .unwrap_or_default(),
        )?;
        Ok(this)
    }
}

#[typetag::serde]
impl Savable for Vec<Countable> {
    fn indexed_db_name(&self) -> String {
        "Countable".into()
    }

    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + 'a>> {
        use wasm_bindgen::JsValue;

        Box::pin(async move {
            for c in self {
                let key = JsValue::from_str(&c.uuid().to_string());
                let value = c.as_js();

                obj.put_kv(&key, &value?).await?;
            }
            Ok(())
        })
    }

    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), leptos::ServerFnError>>>>
    {
        Box::pin(api::update_countable_many(self.clone()))
    }

    fn message(&self) -> Option<leptos::View> {
        None
    }

    fn clone_box(&self) -> Box<dyn Savable> {
        Box::new(self.clone())
    }
    fn has_change(&self) -> bool {
        true
    }
}

#[typetag::serde]
impl Savable for Countable {
    fn indexed_db_name(&self) -> String {
        "Countable".into()
    }

    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + 'a>> {
        use wasm_bindgen::JsValue;
        let key = JsValue::from_str(&self.uuid().to_string());
        let value = self.as_js();
        Box::pin(async move {
            obj.put_kv(&key, &value?).await?;
            Ok(())
        })
    }

    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), leptos::ServerFnError>>>>
    {
        Box::pin(api::update_countable_many(vec![self.clone()]))
    }

    fn message(&self) -> Option<leptos::View> {
        None
    }

    fn clone_box(&self) -> Box<dyn Savable> {
        Box::new(self.clone())
    }

    fn has_change(&self) -> bool {
        true
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
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub parent: Option<CountableId>,
    #[serde(default)]
    pub children: Vec<CountableId>,
    pub name: String,
    pub last_edit: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub is_deleted: bool,
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
        }
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

#[serde_with::serde_as]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase {
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub parent: CountableId,
    pub name: String,
    pub count: i32,
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64>")]
    pub time: chrono::Duration,
    pub hunt_type: Hunttype,
    pub has_charm: bool,
    pub success: bool,
    pub last_edit: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub is_deleted: bool,
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

impl From<Hunttype> for components::SelectOption {
    fn from(val: Hunttype) -> Self {
        (val.repr(), val.into()).into()
    }
}

impl From<Hunttype> for leptos::Attribute {
    fn from(val: Hunttype) -> Self {
        let str: &'static str = val.into();
        leptos::Attribute::String(str.into())
    }
}

impl leptos::IntoAttribute for Hunttype {
    fn into_attribute(self) -> leptos::Attribute {
        self.into()
    }

    fn into_attribute_boxed(self: Box<Self>) -> leptos::Attribute {
        (*self).into()
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
