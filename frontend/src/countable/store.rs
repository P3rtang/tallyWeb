use chrono::TimeDelta;
use leptos::{SignalGetUntracked, SignalUpdateUntracked};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use super::*;

pub trait StoreMethod: Default {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BreathFirst;
impl StoreMethod for BreathFirst {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Level;
impl StoreMethod for Level {}

pub trait StoreCheck: Default {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Checked;
impl StoreCheck for Checked {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UnChecked;
impl StoreCheck for UnChecked {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CountableStore<M: StoreMethod, C: StoreCheck> {
    pub(crate) owner: uuid::Uuid,
    pub(crate) store: HashMap<CountableId, Countable>,
    pub(crate) selection: Vec<CountableId>,
    pub(crate) is_changed: RefCell<bool>,
    phantom_method: std::marker::PhantomData<M>,
    phantom_check: std::marker::PhantomData<C>,
}

impl<M, C> CountableStore<M, C>
where
    M: StoreMethod,
    C: StoreCheck,
{
    pub fn new(owner: uuid::Uuid, store: HashMap<CountableId, Countable>) -> Self {
        Self {
            owner,
            store,
            ..Default::default()
        }
    }

    pub fn owner(&self) -> uuid::Uuid {
        self.owner
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

    pub fn raw_filter(&self, filter: impl Fn(&Countable) -> bool) -> Self {
        let store: HashMap<CountableId, Countable> = self
            .store
            .iter()
            .filter(|(_, b)| filter(b))
            .map(|(a, b)| (*a, b.clone()))
            .collect();

        Self {
            owner: self.owner,
            store,
            selection: self.selection.clone(),
            ..Default::default()
        }
    }

    pub fn root_nodes(&self) -> Vec<Countable> {
        let this: &CountableStore<BreathFirst, Checked> = unsafe { std::mem::transmute(self) };
        this.store
            .values()
            .filter(|v| {
                this.parent(&v.uuid().into())
                    .is_ok_and(|p| p == v.uuid().into())
            })
            .cloned()
            .collect()
    }

    pub fn nodes(&self) -> Vec<Countable> {
        self.store.values().cloned().collect()
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

        self.is_changed.replace(true);

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
        self.is_changed.replace(true);
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

        self.is_changed.replace(true);

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
        self.is_changed.replace(true);
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

        self.is_changed.replace(true);

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

        self.is_changed.replace(true);

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
impl Savable for CountableStore<Level, UnChecked> {
    fn indexed_db_name(&self) -> String {
        "Countable".into()
    }

    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + 'a>> {
        use wasm_bindgen::JsValue;

        self.is_changed.replace(false);

        Box::pin(async move {
            obj.clear().await?;
            for c in self.store.values() {
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
        self.is_changed.replace(false);
        let cloned = self.clone();
        Box::pin(api::update_countable_many(
            cloned.store.into_values().collect(),
        ))
    }

    fn message(&self) -> Option<leptos::View> {
        None
    }

    fn clone_box(&self) -> Box<dyn Savable> {
        Box::new(self.clone())
    }

    fn has_change(&self) -> bool {
        *self.is_changed.borrow()
    }
}

impl<M: StoreMethod> CountableStore<M, UnChecked> {
    pub fn checked(self) -> CountableStore<M, Checked> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn checked_ref(&self) -> &CountableStore<M, Checked> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn checked_mut(&mut self) -> &mut CountableStore<M, Checked> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<M: StoreMethod> CountableStore<M, Checked> {
    pub fn unchecked(self) -> CountableStore<M, UnChecked> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn unchecked_ref(&self) -> &CountableStore<M, UnChecked> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn unchecked_mut(&mut self) -> &mut CountableStore<M, UnChecked> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<C: StoreCheck> CountableStore<Level, C> {
    pub fn recursive(self) -> CountableStore<BreathFirst, C> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn recursive_ref(&self) -> &CountableStore<BreathFirst, C> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn recursive_mut(&mut self) -> &mut CountableStore<BreathFirst, C> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<C: StoreCheck> CountableStore<BreathFirst, C> {
    pub fn level(self) -> CountableStore<Level, C> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn level_ref(&self) -> &CountableStore<Level, C> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn level_mut(&mut self) -> &mut CountableStore<Level, C> {
        unsafe { std::mem::transmute(self) }
    }
}

impl<M: StoreMethod> CountableStore<M, Checked> {
    pub fn merge(&mut self, other: Self) -> Result<(), AppError> {
        for (id, other_c) in other.store {
            if other_c.is_archived() {
                self.store.insert(id, other_c);
            } else if let Some(c) = self.get(&id)
                && (c.last_edit_checked()? > other_c.last_edit_checked()? || c.is_archived())
            {
                continue;
            } else {
                self.store.insert(id, other_c);
            }
        }

        self.is_changed.replace(true);

        Ok(())
    }

    pub fn new_countable(
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

        self.is_changed.replace(true);

        Ok(key)
    }

    pub fn archive(&self, countable: &CountableId) -> Result<Countable, AppError> {
        let this: &CountableStore<BreathFirst, Checked> = unsafe { std::mem::transmute(self) };

        for child in this.children(countable)? {
            this.archive(&child)?;
        }

        match this.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(c) => c.lock()?.is_deleted = true,
            Countable::Phase(p) => p.lock()?.is_deleted = true,
            Countable::Chain(_) => todo!(),
        }

        this.get(countable).ok_or(AppError::CountableNotFound)
    }

    pub fn filter(self, filter: impl Fn(&Countable) -> bool) -> Result<Self, AppError> {
        let mut store = self.raw_filter(filter);

        let keys: Vec<CountableId> = store.store.keys().copied().collect();

        // add back any missing parents
        let this: CountableStore<BreathFirst, Checked> = unsafe { std::mem::transmute(self) };
        for element in keys {
            store.store.extend(
                this.all_parents(&element)?
                    .into_iter()
                    .map(|p| Ok((p, this.get(&p).ok_or(AppError::CountableNotFound)?)))
                    .collect::<Result<HashMap<_, _>, AppError>>()?,
            );
        }

        Ok(store)
    }
}

impl<M: StoreMethod> CountableStore<M, UnChecked> {
    pub fn merge(&mut self, other: Self) {
        self.checked_mut().merge(other.checked()).unwrap()
    }

    pub fn new_countable(
        &mut self,
        name: &str,
        kind: CountableKind,
        parent: Option<CountableId>,
    ) -> CountableId {
        self.checked_mut()
            .new_countable(name, kind, parent)
            .unwrap()
    }

    pub fn filter(self, filter: impl Fn(&Countable) -> bool) -> Self {
        self.checked().filter(filter).unwrap().unchecked()
    }
}

impl CountableStore<Level, Checked> {
    pub fn children(&self, countable: &CountableId) -> Result<Vec<CountableId>, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => c.lock()?.children.clone(),
                _ => Vec::new(),
            },
        )
    }

    pub fn has_child(
        &self,
        countable: &CountableId,
        child: &CountableId,
    ) -> Result<bool, AppError> {
        Ok(self
            .children(countable)?
            .into_iter()
            .map(CountableId::from)
            .collect::<Vec<_>>()
            .contains(child))
    }

    pub fn last_child(&self, countable: &CountableId) -> Result<Option<CountableId>, AppError> {
        Ok(
            match self.get(countable).ok_or(AppError::CountableNotFound)? {
                Countable::Counter(c) => c.lock()?.children.last().copied(),
                Countable::Phase(_) => None,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    pub fn parent(&self, countable: &CountableId) -> Result<Option<CountableId>, AppError> {
        Ok(match self.store.get(countable) {
            Some(Countable::Counter(c)) => c.lock()?.parent,
            Some(Countable::Phase(p)) => Some(p.lock()?.parent),
            Some(Countable::Chain(_)) => todo!(),
            None => None,
        })
    }
}

impl CountableStore<BreathFirst, Checked> {
    pub fn children(&self, countable: &CountableId) -> Result<Vec<CountableId>, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    let mut children = c.lock()?.children.clone();
                    for child in children.clone().iter() {
                        children.append(&mut self.children(child)?)
                    }
                    children
                }
                _ => Vec::new(),
            },
        )
    }

    pub fn has_child(
        &self,
        countable: &CountableId,
        child: &CountableId,
    ) -> Result<bool, AppError> {
        Ok(self
            .children(countable)?
            .into_iter()
            .map(CountableId::from)
            .collect::<Vec<_>>()
            .contains(child))
    }

    /**
        `Recursive Last Child Checked`

        Returns the `last child` of the given `CountableId` recursively and will always return a `leaf node`.
        If the node is not a leaf node it will go down the tree to its children and pick the last one.
        When the element has no children it returns `CountableId` of the element itself

        This function will return an `error` when `CountableId` is not available in the store
        And will return an `error` when any lock on a `Mutex` fails

        `last child`: for all children sorted pick the last one in the array
        `leaf node`: a node without children

        [CountableId]
    */
    pub fn last_child(&self, countable: &CountableId) -> Result<CountableId, AppError> {
        Ok(
            match self.get(countable).ok_or(AppError::CountableNotFound)? {
                Countable::Counter(c) => {
                    if let Some(last) = c.lock()?.children.last().copied() {
                        self.last_child(&last)?
                    } else {
                        *countable
                    }
                }
                Countable::Phase(_) => *countable,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Recursive Parent Checked`

        Returns the parent `root element` of the given `CountableId`
        When the element is a root node it returns the `CountableId` of the element itself

        This function will return an `error` when `CountableId` is not available in the store
        And will return an `error` when any lock on a `Mutex` fails

        `root element`: An element without a parent

        [CountableId]
    */
    pub fn parent(&self, countable: &CountableId) -> Result<CountableId, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    if let Some(parent) = c.lock()?.parent {
                        self.parent(&parent)?
                    } else {
                        *countable
                    }
                }
                Countable::Phase(p) => self.parent(&p.lock()?.parent)?,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `All Parents Checked`

        Returns all parent elements up to the `root element` of the given `CountableId`
        excluding the element itself

        This function will return an `error` when `CountableId` is not available in the store
        And will return an `error` when any lock on a `Mutex` fails

        `root element`: An element without a parent

        [CountableId]
    */
    pub fn all_parents(&self, countable: &CountableId) -> Result<Vec<CountableId>, AppError> {
        fn all_parents_rec<'a>(
            store: &'a CountableStore<BreathFirst, Checked>,
            countable: &'a CountableId,
            list: &'a mut Vec<CountableId>,
        ) -> Result<&'a mut Vec<CountableId>, AppError> {
            list.push(*countable);
            if let Some(parent) = store.level_ref().parent(countable)? {
                all_parents_rec(store, &parent, list)?;
            };
            Ok(list)
        }

        let mut list = Vec::new();

        if let Some(parent) = self.level_ref().parent(countable)? {
            all_parents_rec(self, &parent, &mut list)?;
        };

        Ok(list)
    }
}

impl CountableStore<Level, UnChecked> {
    pub fn children(&self, countable: &CountableId) -> Vec<CountableId> {
        self.clone()
            .checked()
            .children(countable)
            .unwrap_or_default()
    }

    pub fn has_child(&self, countable: &CountableId, child: &CountableId) -> bool {
        self.clone()
            .checked()
            .has_child(countable, child)
            .unwrap_or_default()
    }

    pub fn archive(&self, countable: &CountableId) -> Option<Countable> {
        self.checked_ref().archive(countable).ok()
    }

    pub fn last_child(&self, countable: &CountableId) -> Option<CountableId> {
        self.checked_ref().last_child(countable).unwrap()
    }

    pub fn parent(&self, countable: &CountableId) -> Option<CountableId> {
        self.checked_ref().parent(countable).unwrap_or_default()
    }
}

impl CountableStore<BreathFirst, UnChecked> {
    pub fn children(&self, countable: &CountableId) -> Vec<CountableId> {
        self.clone()
            .checked()
            .children(countable)
            .unwrap_or_default()
    }

    pub fn has_child(&self, countable: &CountableId, child: &CountableId) -> bool {
        self.clone()
            .checked()
            .has_child(countable, child)
            .unwrap_or_default()
    }

    pub fn archive(&self, countable: &CountableId) -> Option<Countable> {
        self.checked_ref().archive(countable).ok()
    }

    /**
        `Recursive Last Child`

        Returns the `last child` of the given `CountableId` recursively and will always return a `leaf node`.
        If the node is not a leaf node it will go down the tree to its children and pick the last one.
        When the element has no children it returns `CountableId` of the element itself

        This function will return the given `CountableId` when it is not available in the store
        And will `panic` when any lock on a `Mutex` fails

        `last child`: for all children sorted pick the last one in the array
        `leaf node`: a node without children

        [CountableId]
    */
    pub fn last_child(&self, countable: &CountableId) -> CountableId {
        match self.checked_ref().last_child(countable) {
            Ok(c) => c,
            Err(AppError::CountableNotFound) => *countable,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Parent`

        Returns the parent `root element` of the given `CountableId`
        When the element is a root node it returns the `CountableId` of the element itself

        This function will return the given `CountableId` when it is not available in the store
        And will `panic` when any lock on a `Mutex` fails

        `root element`: An element without a parent

        [CountableId]
    */
    pub fn parent(&self, countable: &CountableId) -> CountableId {
        match self.checked_ref().parent(countable) {
            Ok(c) => c,
            Err(AppError::CountableNotFound) => *countable,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `All Parents`

        Returns all parent elements up to the `root element` of the given `CountableId`
        excluding the element itself

        This function will return an empty `Vec` when `CountableId` is not available in the store
        And will `panic` when any lock on a `Mutex` fails

        `root element`: An element without a parent

        [CountableId]
    */
    pub fn all_parents(&self, countable: &CountableId) -> Vec<CountableId> {
        match self.checked_ref().all_parents(countable) {
            Ok(l) => l,
            Err(AppError::CountableNotFound) => Vec::new(),
            Err(err) => panic!("{}", err),
        }
    }
}

#[typetag::serde]
impl Savable for leptos::RwSignal<CountableStore<Level, UnChecked>> {
    fn indexed_db_name(&self) -> String {
        "Countable".into()
    }

    fn save_indexed<'a>(
        &'a self,
        obj: indexed_db::ObjectStore<AppError>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + 'a>> {
        use wasm_bindgen::JsValue;

        self.update_untracked(|s| {
            s.is_changed.replace(false);
        });

        Box::pin(async move {
            obj.clear().await?;
            for c in self.get_untracked().store.values() {
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
        self.update_untracked(|s| {
            s.is_changed.replace(false);
        });

        Box::pin(api::update_countable_many(
            self.get_untracked().store.into_values().collect(),
        ))
    }

    fn message(&self) -> Option<leptos::View> {
        None
    }

    fn clone_box(&self) -> Box<dyn Savable> {
        Box::new(*self)
    }

    fn has_change(&self) -> bool {
        *self.get_untracked().is_changed.borrow()
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
