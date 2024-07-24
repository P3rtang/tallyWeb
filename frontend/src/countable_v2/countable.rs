use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CountableStore {
    owner: uuid::Uuid,
    store: HashMap<CountableId, Countable>,
    selection: Vec<CountableId>,
}

impl CountableStore {
    pub fn new(owner: uuid::Uuid, store: HashMap<CountableId, Countable>) -> Self {
        Self {
            owner,
            store,
            ..Default::default()
        }
    }

    pub fn contains(&self, countable: &CountableId) -> bool {
        self.store.contains_key(countable)
    }

    pub fn get(&self, countable: &CountableId) -> Option<Countable> {
        self.store.get(countable).cloned()
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn new_countable_checked(
        &mut self,
        name: &str,
        kind: CountableKind,
        parent: Option<CountableId>,
    ) -> Result<CountableId, AppError> {
        let countable = Countable::new(name, kind, self.owner, parent);
        let key = countable.clone().into();
        self.store.insert(key, countable);
        if let Some(parent) = parent {
            self.get(&parent)
                .ok_or(AppError::CountableNotFound)?
                .add_child_checked(key)?
        }
        Ok(key)
    }

    pub fn new_countable(
        &mut self,
        name: &str,
        kind: CountableKind,
        parent: Option<CountableId>,
    ) -> CountableId {
        self.new_countable_checked(name, kind, parent).unwrap()
    }

    pub fn root_nodes(&self) -> Vec<Countable> {
        self.store
            .values()
            .filter(|v| self.parent(&v.uuid().into()).is_none())
            .cloned()
            .collect()
    }

    pub fn children_checked(&self, countable: &CountableId) -> Result<Vec<Countable>, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    let children = c.lock()?.children.clone();
                    children
                        .into_iter()
                        .filter_map(|id| self.store.get(&id).cloned())
                        .collect()
                }
                _ => Vec::new(),
            },
        )
    }

    pub fn children(&self, countable: &CountableId) -> Vec<Countable> {
        self.children_checked(countable).unwrap()
    }

    pub fn parent_checked(&self, countable: &CountableId) -> Result<Option<Countable>, AppError> {
        Ok(match self.store.get(countable) {
            Some(Countable::Counter(c)) => {
                c.lock()?.parent.and_then(|id| self.store.get(&id)).cloned()
            }
            Some(Countable::Phase(p)) => self.store.get(&p.lock()?.parent).cloned(),
            Some(Countable::Chain(_)) => todo!(),
            None => None,
        })
    }

    pub fn parent(&self, countable: &CountableId) -> Option<Countable> {
        self.parent_checked(countable).unwrap()
    }

    pub fn kind_checked(&self, countable: &CountableId) -> Result<CountableKind, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(_) => CountableKind::Counter,
                Countable::Phase(_) => CountableKind::Phase,
                Countable::Chain(_) => CountableKind::Chain,
            },
        )
    }

    pub fn kind(&self, countable: &CountableId) -> CountableKind {
        self.kind_checked(countable).unwrap()
    }

    pub fn name_checked(&self, countable: &CountableId) -> Result<String, AppError> {
        self.store
            .get(countable)
            .ok_or(AppError::CountableNotFound)?
            .name_checked()
    }

    pub fn name(&self, countable: &CountableId) -> String {
        self.name_checked(countable).unwrap()
    }

    pub fn set_name_checked(&self, countable: &CountableId, name: &str) -> Result<(), AppError> {
        match self
            .store
            .get(countable)
            .ok_or(AppError::CountableNotFound)?
        {
            Countable::Counter(c) => c.lock()?.name = name.into(),
            Countable::Phase(p) => p.lock()?.name = name.into(),
            Countable::Chain(_) => todo!(),
        };
        Ok(())
    }

    pub fn set_name(&self, countable: &CountableId, name: &str) {
        self.set_name_checked(countable, name).unwrap()
    }

    pub fn count_checked(&self, countable: &CountableId) -> Result<i32, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    let mut sum = 0;
                    for child in c.lock()?.children.iter() {
                        sum += self.count_checked(child)?;
                    }
                    sum
                }
                Countable::Phase(p) => p.lock()?.count,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    pub fn count(&self, countable: &CountableId) -> i32 {
        self.count_checked(countable).unwrap()
    }

    pub fn set_count_checked(&self, countable: &CountableId, count: i32) -> Result<(), AppError> {
        let diff = count - self.count_checked(countable)?;
        self.add_count_checked(countable, diff)?;
        Ok(())
    }

    pub fn set_count(&self, countable: &CountableId, count: i32) {
        self.set_count_checked(countable, count).unwrap()
    }

    pub fn add_count_checked(&self, countable: &CountableId, mut add: i32) -> Result<(), AppError> {
        match self
            .store
            .get(countable)
            .ok_or(AppError::CountableNotFound)?
        {
            Countable::Counter(c) => {
                for child in c.lock()?.children.iter().rev() {
                    let child_count = self.count_checked(child)?;
                    if child_count + add <= 0 {
                        self.set_count_checked(child, 0)?;
                        add += self.count_checked(child)?;
                    } else {
                        self.set_count_checked(child, child_count + add)?;
                        return Ok(());
                    }
                }
            }
            Countable::Phase(p) => {
                p.lock()?.count += add;
            }
            Countable::Chain(_) => todo!(),
        }
        Ok(())
    }

    pub fn add_count(&self, countable: &CountableId, add: i32) {
        self.add_count_checked(countable, add).unwrap();
    }

    pub fn time_checked(&self, countable: &CountableId) -> Result<TimeDelta, AppError> {
        match self
            .store
            .get(countable)
            .ok_or(AppError::CountableNotFound)?
        {
            Countable::Counter(c) => {
                let mut time = TimeDelta::zero();
                for child in c.lock()?.children.iter() {
                    time += self.time_checked(child)?;
                }
                Ok(time)
            }
            Countable::Phase(p) => Ok(p.lock()?.time),
            Countable::Chain(_) => todo!(),
        }
    }

    pub fn time(&self, countable: &CountableId) -> TimeDelta {
        self.time_checked(countable).unwrap()
    }

    pub fn set_time_checked(
        &self,
        countable: &CountableId,
        time: TimeDelta,
    ) -> Result<(), AppError> {
        let diff = time - self.time_checked(countable)?;
        self.add_time_checked(countable, diff)?;
        Ok(())
    }

    pub fn set_time(&self, countable: &CountableId, time: TimeDelta) {
        self.set_time_checked(countable, time).unwrap()
    }

    pub fn add_time_checked(
        &self,
        countable: &CountableId,
        mut add: TimeDelta,
    ) -> Result<(), AppError> {
        match self
            .store
            .get(countable)
            .ok_or(AppError::CountableNotFound)?
        {
            Countable::Counter(c) => {
                for child in c.lock()?.children.iter().rev() {
                    let child_time = self.time_checked(child)?;
                    if child_time + add <= TimeDelta::zero() {
                        self.set_time_checked(child, TimeDelta::zero())?;
                        add += self.time_checked(child)?;
                    } else {
                        self.set_time_checked(child, child_time + add)?;
                    }
                }
            }
            Countable::Phase(p) => {
                p.lock()?.time += add;
            }
            Countable::Chain(_) => todo!(),
        }
        Ok(())
    }

    pub fn add_time(&self, countable: &CountableId, add: TimeDelta) {
        self.add_time_checked(countable, add).unwrap();
    }

    pub fn hunttype_checked(&self, countable: &CountableId) -> Result<Hunttype, AppError> {
        match self
            .store
            .get(countable)
            .ok_or(AppError::CountableNotFound)?
        {
            Countable::Counter(c) => {
                let children = c.lock()?.children.clone();

                let ht = children
                    .first()
                    .and_then(|child| self.hunttype_checked(child).ok())
                    .unwrap_or_default();

                for child in children.iter() {
                    if self.hunttype_checked(child)? != ht {
                        return Ok(Hunttype::Mixed);
                    }
                }
                Ok(ht)
            }
            Countable::Phase(p) => Ok(p.lock()?.hunt_type),
            Countable::Chain(_) => todo!(),
        }
    }

    pub fn hunttype(&self, countable: &CountableId) -> Hunttype {
        self.hunttype_checked(countable).unwrap()
    }

    pub fn rolls_checked(&self, countable: &CountableId) -> Result<usize, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => c
                    .lock()?
                    .children
                    .iter()
                    .map(|child| self.rolls_checked(child))
                    .collect::<Result<Vec<_>, AppError>>()?
                    .into_iter()
                    .sum(),
                Countable::Phase(_) => self.hunttype_checked(countable)?.rolls()(
                    self.count_checked(countable)?,
                    self.has_charm_checked(countable)?,
                ),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    pub fn rolls(&self, countable: &CountableId) -> usize {
        self.rolls_checked(countable).unwrap()
    }

    pub fn odds_checked(&self, countable: &CountableId) -> Result<f64, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    let sum = c
                        .lock()?
                        .children
                        .iter()
                        .map(|child| {
                            let odds = self.odds_checked(child)?;
                            Ok(odds * self.count_checked(child)? as f64)
                        })
                        .collect::<Result<Vec<_>, AppError>>()?
                        .into_iter()
                        .sum::<f64>();
                    sum / (self.count_checked(countable)? as f64).max(1.0)
                }
                Countable::Phase(p) => p.lock()?.hunt_type.odds(),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    pub fn odds(&self, countable: &CountableId) -> f64 {
        self.odds_checked(countable).unwrap()
    }

    pub fn progress_checked(&self, countable: &CountableId) -> Result<f64, AppError> {
        let prob = 1.0 / self.odds_checked(countable)?;
        let rolls = self.rolls_checked(countable)?;
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    let children_len = c.lock()?.children.len();
                    let mut chance = 0.0;
                    for k in 0..((self.completed_checked(countable)? + 1).min(children_len)) {
                        let combs = n_choose_k(rolls, k);
                        chance +=
                            combs * prob.powi(k as i32) * (1.0 - prob).powi((rolls - k) as i32)
                    }

                    1.0 - chance
                }
                Countable::Phase(_) => 1.0 - (1.0 - prob).powi(rolls as i32),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    pub fn progress(&self, countable: &CountableId) -> f64 {
        self.progress_checked(countable).unwrap()
    }

    pub fn completed_checked(&self, countable: &CountableId) -> Result<usize, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => c
                    .lock()?
                    .children
                    .iter()
                    .map(|child| self.completed_checked(child))
                    .collect::<Result<Vec<_>, AppError>>()?
                    .into_iter()
                    .sum(),
                Countable::Phase(p) => p.lock()?.success.into(),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    pub fn has_charm_checked(&self, countable: &CountableId) -> Result<bool, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    let mut has = true;
                    for child in c.lock()?.children.iter() {
                        has &= self.has_charm_checked(child)?;
                    }
                    has
                }
                Countable::Phase(p) => p.lock()?.has_charm,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    pub fn has_charm(&self, countable: &CountableId) -> bool {
        self.has_charm_checked(countable).unwrap()
    }

    pub fn is_success_checked(&self, countable: &CountableId) -> Result<bool, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => c
                    .lock()?
                    .children
                    .last()
                    .and_then(|child| self.is_success_checked(child).ok())
                    .unwrap_or_default(),
                Countable::Phase(p) => p.lock()?.success,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    pub fn is_success(&self, countable: &CountableId) -> bool {
        self.is_success_checked(countable).unwrap()
    }

    pub fn toggle_success_checked(&self, countable: &CountableId) -> Result<(), AppError> {
        match self
            .store
            .get(countable)
            .ok_or(AppError::CountableNotFound)?
        {
            Countable::Counter(_) => {}
            Countable::Phase(p) => {
                let success = p.lock()?.success;
                p.lock()?.success = !success;
            }
            Countable::Chain(_) => todo!(),
        };
        Ok(())
    }

    pub fn toggle_success(&self, countable: &CountableId) {
        self.toggle_success_checked(countable).unwrap()
    }

    pub fn created_at_checked(
        &self,
        countable: &CountableId,
    ) -> Result<chrono::NaiveDateTime, AppError> {
        self.store
            .get(countable)
            .ok_or(AppError::CountableNotFound)?
            .created_at_checked()
    }

    pub fn created_at(&self, countable: &CountableId) -> chrono::NaiveDateTime {
        self.created_at_checked(countable).unwrap()
    }
}

#[typetag::serde]
impl Savable for CountableStore {
    fn indexed_db_name(&self) -> String {
        "CountableStore".into()
    }

    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), leptos::ServerFnError>>>>
    {
        let cloned = self.clone();
        Box::pin(api::update_countable_many(
            cloned.store.into_values().collect(),
        ))
    }
}

#[typetag::serde]
impl Savable for Vec<Countable> {
    fn indexed_db_name(&self) -> String {
        "CountableStore".into()
    }

    fn save_endpoint(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), leptos::ServerFnError>>>>
    {
        Box::pin(api::update_countable_many(self.clone()))
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct CountableId(uuid::Uuid);

impl From<uuid::Uuid> for CountableId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
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
            created_at: value.created_at,
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
            created_at: value.created_at,
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
    pub created_at: chrono::NaiveDateTime,
}

impl Counter {
    fn new(name: String, owner_uuid: uuid::Uuid, parent: Option<CountableId>) -> Self {
        Self {
            uuid: uuid::Uuid::new_v4(),
            owner_uuid,
            parent,
            children: Vec::new(),
            name,
            created_at: chrono::Utc::now().naive_utc(),
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
            created_at: self.created_at,
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
    pub created_at: chrono::NaiveDateTime,
}

impl Phase {
    fn new(name: String, owner_uuid: uuid::Uuid, parent: CountableId) -> Self {
        Self {
            uuid: uuid::Uuid::new_v4(),
            owner_uuid,
            parent,
            name,
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
            created_at: self.created_at,
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
    fn rolls(&self) -> impl Fn(i32, bool) -> usize {
        match self {
            Hunttype::OldOdds => {
                |count, has_charm: bool| (count * if has_charm { 3 } else { 1 }) as usize
            }
            Hunttype::NewOdds => {
                |count, has_charm: bool| (count * if has_charm { 3 } else { 1 }) as usize
            }
            Hunttype::SOS => |count, has_charm: bool| match count {
                c if c < 10 => (count * if has_charm { 3 } else { 1 }) as usize,
                c if c < 20 => (10 + (count - 10) * if has_charm { 3 + 4 } else { 1 + 4 }) as usize,
                c if c < 30 => (60 + (count - 20) * if has_charm { 3 + 8 } else { 1 + 8 }) as usize,
                _ => (50 + (count - 30) * if has_charm { 3 + 13 } else { 1 + 12 }) as usize,
            },
            Hunttype::Masuda(Masuda::GenIV) => {
                |count, has_charm: bool| (count * if has_charm { 3 + 4 } else { 1 + 4 }) as usize
            }
            Hunttype::Masuda(_) => {
                |count, has_charm: bool| (count * if has_charm { 3 + 5 } else { 1 + 5 }) as usize
            }
            Hunttype::Mixed => unreachable!(),
        }
    }

    fn odds(&self) -> f64 {
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
            Self::Mixed => todo!(),
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
            Hunttype::Mixed => todo!(),
        }
    }
}

impl TryFrom<String> for Hunttype {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        return match value.as_str() {
            "OldOdds" => Ok(Self::OldOdds),
            "NewOdds" => Ok(Self::NewOdds),
            "SOS" => Ok(Self::SOS),
            "MasudaGenIV" => Ok(Self::Masuda(Masuda::GenIV)),
            "MasudaGenV" => Ok(Self::Masuda(Masuda::GenV)),
            "MasudaGenVI" => Ok(Self::Masuda(Masuda::GenVI)),
            _ => Err(String::from(
                "Hunttype should be one of the following: OldOdds, NewOdds, SOS, Masuda",
            )),
        };
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

fn n_choose_k(n: usize, k: usize) -> f64 {
    match (n, k) {
        (n, k) if k > n => 0.0,
        (_, 0) => 1.0,
        (n, k) if k > n / 2 => n_choose_k(n, n - k),
        (n, k) => n as f64 / k as f64 * n_choose_k(n - 1, k - 1),
    }
}
