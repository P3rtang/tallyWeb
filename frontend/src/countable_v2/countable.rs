use super::AppError;
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use serde_with;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CountableStore {
    pub(crate) store: HashMap<Countable, CountableKind>,
}

impl CountableStore {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Countable(uuid::Uuid);

impl From<uuid::Uuid> for Countable {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CountableKind {
    Counter(Arc<Mutex<Counter>>),
    Phase(Arc<Mutex<Phase>>),
    Chain(Arc<Mutex<Chain>>),
}

impl CountableKind {
    pub fn add_child(self: &Self, child: CountableKind) -> Result<(), AppError> {
        match self {
            CountableKind::Counter(c) => {
                c.try_lock()?.children.push(child);
                Ok(())
            }
            CountableKind::Phase(_) => Err(AppError::CannotContainChildren("Phase".into())),
            CountableKind::Chain(_) => Err(AppError::CannotContainChildren("Chain".into())),
        }
    }

    pub fn name_checked(self: &Self) -> Result<String, AppError> {
        Ok(match self {
            CountableKind::Counter(c) => c.lock()?.name.clone(),
            CountableKind::Phase(p) => p.lock()?.name.clone(),
            CountableKind::Chain(_) => todo!(),
        })
    }

    pub fn name(self: &Self) -> String {
        self.name_checked().unwrap()
    }

    pub fn set_name_checked(self: &Self, name: &str) -> Result<(), AppError> {
        match self {
            CountableKind::Counter(c) => c.lock()?.name = name.into(),
            CountableKind::Phase(p) => p.lock()?.name = name.into(),
            CountableKind::Chain(_) => todo!(),
        }

        Ok(())
    }

    pub fn set_name(self: &Self, name: &str) {
        self.set_name_checked(name).unwrap()
    }

    pub fn count_checked(self: &Self) -> Result<i32, AppError> {
        match self {
            CountableKind::Counter(c) => {
                let children = c.lock()?.children.clone();
                let mut sum = 0;
                for child in children {
                    sum += child.count_checked()?;
                }
                Ok(sum)
            }
            CountableKind::Phase(p) => Ok(p.lock()?.count),
            CountableKind::Chain(_) => todo!(),
        }
    }

    pub fn count(self: &Self) -> i32 {
        self.count_checked().unwrap()
    }

    pub fn set_count_checked(self: &Self, count: i32) -> Result<(), AppError> {
        let diff = count - self.count_checked()?;
        self.add_count_checked(diff)?;
        Ok(())
    }

    pub fn set_count(self: &Self, count: i32) {
        self.set_count_checked(count).unwrap()
    }

    pub fn add_count_checked(self: &Self, mut add: i32) -> Result<(), AppError> {
        match self {
            CountableKind::Counter(c) => {
                let children = c.lock()?.children.clone();
                for child in children {
                    let child_count = child.count_checked()?;
                    if child_count + add <= 0 {
                        child.set_count_checked(0)?;
                        add += child.count_checked()?;
                    } else {
                        child.set_count_checked(child_count + add)?;
                    }
                }
            }
            CountableKind::Phase(p) => {
                p.lock()?.count += add;
            }
            CountableKind::Chain(_) => todo!(),
        }
        Ok(())
    }

    pub fn add_count(self: &Self, add: i32) {
        self.add_count_checked(add).unwrap();
    }

    pub fn time_checked(self: &Self) -> Result<TimeDelta, AppError> {
        match self {
            CountableKind::Counter(c) => {
                let children = c.lock()?.children.clone();
                let mut time = TimeDelta::zero();
                for child in children {
                    time += child.time_checked()?;
                }
                Ok(time)
            }
            CountableKind::Phase(p) => Ok(p.lock()?.time),
            CountableKind::Chain(_) => todo!(),
        }
    }

    pub fn time(self: &Self) -> TimeDelta {
        self.time_checked().unwrap()
    }

    pub fn set_time_checked(self: &Self, time: TimeDelta) -> Result<(), AppError> {
        let diff = time - self.time_checked()?;
        self.add_time_checked(diff)?;
        Ok(())
    }

    pub fn set_time(self: &Self, count: TimeDelta) {
        self.set_time_checked(count).unwrap()
    }

    pub fn add_time_checked(self: &Self, mut add: TimeDelta) -> Result<(), AppError> {
        match self {
            CountableKind::Counter(c) => {
                let children = c.lock()?.children.clone();
                for child in children {
                    let child_time = child.time_checked()?;
                    if child_time + add <= TimeDelta::zero() {
                        child.set_time_checked(TimeDelta::zero())?;
                        add += child.time_checked()?;
                    } else {
                        child.set_time_checked(child_time + add)?;
                    }
                }
            }
            CountableKind::Phase(p) => {
                p.lock()?.time += add;
            }
            CountableKind::Chain(_) => todo!(),
        }
        Ok(())
    }

    pub fn add_time(self: &Self, add: TimeDelta) {
        self.add_time_checked(add).unwrap();
    }

    pub fn hunttype_checked(self: &Self) -> Result<Hunttype, AppError> {
        match self {
            CountableKind::Counter(c) => {
                let children = c.lock()?.children.clone();
                if children.len() == 0 {
                    return Ok(Default::default());
                }
                let ht = children.first().unwrap().hunttype_checked()?;
                for child in children {
                    if child.hunttype_checked()? != ht {
                        return Ok(Hunttype::Mixed);
                    }
                }
                return Ok(ht);
            }
            CountableKind::Phase(p) => Ok(p.lock()?.hunt_type),
            CountableKind::Chain(_) => todo!(),
        }
    }

    pub fn hunttype(self: &Self) -> Hunttype {
        self.hunttype_checked().unwrap()
    }

    pub fn has_charm_checked(self: &Self) -> Result<bool, AppError> {
        match self {
            CountableKind::Counter(c) => {
                let mut has = true;
                let children = c.lock()?.children.clone();
                for child in children {
                    has &= child.has_charm_checked()?;
                }
                Ok(has)
            }
            CountableKind::Phase(p) => Ok(p.lock()?.has_charm),
            CountableKind::Chain(_) => todo!(),
        }
    }

    pub fn has_charm(self: &Self) -> bool {
        self.has_charm_checked().unwrap()
    }
}

impl PartialEq for CountableKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CountableKind::Counter(a), CountableKind::Counter(b)) => {
                match (a.try_lock(), b.try_lock()) {
                    (Ok(a), Ok(b)) => a.eq(&*b),
                    _ => false,
                }
            }
            (CountableKind::Phase(a), CountableKind::Phase(b)) => {
                match (a.try_lock(), b.try_lock()) {
                    (Ok(a), Ok(b)) => a.eq(&*b),
                    _ => false,
                }
            }
            (CountableKind::Chain(a), CountableKind::Chain(b)) => {
                match (a.try_lock(), b.try_lock()) {
                    (Ok(a), Ok(b)) => a.eq(&*b),
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl Eq for CountableKind {}

#[cfg(feature = "ssr")]
impl From<backend::DbCounter> for CountableKind {
    fn from(value: backend::DbCounter) -> Self {
        Self::Counter(Arc::new(Mutex::new(Counter {
            uuid: value.uuid,
            owner_uuid: value.owner_uuid,
            parent_uuid: None,
            children: Vec::new(),
            name: value.name,
            created_at: value.created_at,
        })))
    }
}

#[cfg(feature = "ssr")]
impl From<backend::DbPhase> for CountableKind {
    fn from(value: backend::DbPhase) -> Self {
        Self::Phase(Arc::new(Mutex::new(Phase {
            uuid: value.uuid,
            owner_uuid: value.owner_uuid,
            parent_uuid: value.parent_uuid,
            name: value.name,
            count: value.count,
            time: chrono::Duration::milliseconds(value.time),
            hunt_type: value.hunt_type.into(),
            has_charm: value.has_charm,
            success: value.success,
            created_at: value.created_at,
        })))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counter {
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub parent_uuid: Option<uuid::Uuid>,
    pub children: Vec<CountableKind>,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase {
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub parent_uuid: uuid::Uuid,
    pub name: String,
    pub count: i32,
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64>")]
    pub time: chrono::Duration,
    pub hunt_type: Hunttype,
    pub has_charm: bool,
    pub success: bool,
    pub created_at: chrono::NaiveDateTime,
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
