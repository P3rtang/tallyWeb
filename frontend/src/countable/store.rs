use chrono::TimeDelta;
use leptos::{SignalGetUntracked, SignalUpdateUntracked};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use super::*;

pub trait StoreMethod: Default {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Recursive;
impl StoreMethod for Recursive {}

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
    phantom_data: std::marker::PhantomData<(M, C)>,
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

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
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
        let this: &CountableStore<Recursive, Checked> = unsafe { std::mem::transmute(self) };
        this.store
            .values()
            .filter(|v| {
                this.root_parent(&v.uuid().into())
                    .is_ok_and(|p| p == v.uuid().into())
            })
            .cloned()
            .collect()
    }

    pub fn nodes(&self) -> Vec<Countable> {
        self.store.values().cloned().collect()
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

/// Methods to transform `CountableStore` into `Checked` mode
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

/// Methods to transform `CountableStore` into `UnChecked` mode
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

/// Methods to transform `CountableStore` into `Recursive` mode
impl<C: StoreCheck> CountableStore<Level, C> {
    pub fn recursive(self) -> CountableStore<Recursive, C> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn recursive_ref(&self) -> &CountableStore<Recursive, C> {
        unsafe { std::mem::transmute(self) }
    }

    pub fn recursive_mut(&mut self) -> &mut CountableStore<Recursive, C> {
        unsafe { std::mem::transmute(self) }
    }
}

/// Methods to transform `CountableStore` into `Level` mode
impl<C: StoreCheck> CountableStore<Recursive, C> {
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
        let this: &CountableStore<Recursive, Checked> = unsafe { std::mem::transmute(self) };

        for child in this.children(countable)? {
            this.archive(&child)?;
        }

        match this.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(c) => c.lock()?.is_deleted = true,
            Countable::Phase(p) => p.lock()?.is_deleted = true,
            Countable::Chain(_) => todo!(),
        }

        self.is_changed.replace(true);

        this.get(countable).ok_or(AppError::CountableNotFound)
    }

    /**
        `CountableStore Filter Checked`

        # Description

        Filter allows filtering tree elements with a `filter` callback.
        The elements remaining in the store will only be:\
          * those returning true with the `filter` function
          * their parents up to the `root element`

        Use `raw_filter` to return a store with only elements matching `filter` and not their parents

        # Arguments
          * `filter`: `impl Fn(&Countable) -> bool`

        # Returns
          * `Ok(CountableStore<M, Checked>)`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn filter(self, filter: impl Fn(&Countable) -> bool) -> Result<Self, AppError> {
        let mut store = self.raw_filter(filter);

        let keys: Vec<CountableId> = store.store.keys().copied().collect();

        // add back any missing parents
        let this: CountableStore<Recursive, Checked> = unsafe { std::mem::transmute(self) };
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

    /**
        `Countable Kind Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(CountableKind)` for the given `CountableId`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [CountableKind]\
        [AppError]
    */
    pub fn kind(&self, countable: &CountableId) -> Result<CountableKind, AppError> {
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

    /**
        `Countable Name Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(String)`: The name of the `Countable` for the given `CountableId`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn name(&self, countable: &CountableId) -> Result<String, AppError> {
        self.get(countable)
            .ok_or(AppError::CountableNotFound)?
            .name_checked()
    }

    /**
        `Set Countable Name Checked`

        # Arguments
          * `countable`: &[CountableId]
          * `name`: &str; The new name for the `Countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn set_name(&self, countable: &CountableId, name: &str) -> Result<(), AppError> {
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

    /**
        `CountableStore Filter UnChecked`

        # Description

        Filter allows filtering tree elements with a `filter` callback.
        The elements remaining in the store will only be:\
          * those returning true with the `filter` function
          * their parents up to the `root element`

        Use `raw_filter` to return a store with only elements matching `filter` and not their parents

        # Arguments
          * `filter`: `impl Fn(&Countable) -> bool`

        # Returns
          * `CountableStore<M, UnChecked>`
          * `Err(AppError)`

        # Panics
          * any parent elements are not available in the store
          * lock on a `Mutex` fails

        To use a version that does not panic use the `Checked` version of `CountableStore` instread

        [Countable]
    */
    pub fn filter(self, filter: impl Fn(&Countable) -> bool) -> Self {
        self.checked().filter(filter).unwrap().unchecked()
    }

    /**
        `Countable Kind UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        #  Returns
          * `CountableKind` for the given `CountableId`

        # Panics
          * `CountableId` is not available in the store
          * lock on a `Mutex` fails

        To use a version that does not panic use the `Checked` version of `CountableStore` instread

        [CountableKind]
    */
    pub fn kind(&self, countable: &CountableId) -> CountableKind {
        match self.checked_ref().kind(countable) {
            Ok(kind) => kind,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Countable Name UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `String`: The name for the `Countable` for the given `CountableId`
          * empty `String`: when the `CountableId` was not in the `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        To use a version that does not panic use the `Checked` version of `CountableStore` instread

        [Countable]
    */
    pub fn name(&self, countable: &CountableId) -> String {
        match self
            .get(countable)
            .ok_or(AppError::CountableNotFound)
            .and_then(|c| c.name_checked())
        {
            Ok(name) => name,
            Err(AppError::CountableNotFound) => String::new(),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Set Countable Name UnChecked`

        # Arguments
          * `countable`: &[CountableId]
          * `name`: &str; The new name for the `Countable`

        # Panics
          * lock on a `Mutex` fails

        [Countable]
    */
    pub fn set_name(&self, countable: &CountableId, name: &str) {
        match self.checked_ref().set_name(countable, name) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        };

        self.is_changed.replace(true);
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

    /**
        `Countable Count Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(i32)`: The count of the `Countable` for the given `CountableId`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn count(&self, countable: &CountableId) -> Result<i32, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(_) => 0,
                Countable::Phase(p) => p.lock()?.count,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Set Countable Count Checked`

        Since this function does not recurse
        it will only change elements that hold a count value themselves not any descendants

        # Arguments
          * `countable`: &[CountableId]
          * `count`: i32; The new count for the `Countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn set_count(&self, countable: &CountableId, count: i32) -> Result<(), AppError> {
        match self.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(_) => (),
            Countable::Phase(p) => p.lock()?.count = count,
            Countable::Chain(_) => todo!(),
        };

        self.is_changed.replace(true);

        Ok(())
    }

    /**
        `Add Count Checked`

        Since this function does not recurse
        it will only change elements that hold a count value themselves not any descendants

        # Arguments
          * `countable`: &[CountableId]
          * `count`: i32; The count to add to `countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn add_count(&self, countable: &CountableId, count: i32) -> Result<(), AppError> {
        match self.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(_) => (),
            Countable::Phase(p) => p.lock()?.count += count,
            Countable::Chain(_) => todo!(),
        };

        self.is_changed.replace(true);

        Ok(())
    }

    /**
        `Countable Time Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(TimeDelta)`: The time of the `Countable` for the given `CountableId`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn time(&self, countable: &CountableId) -> Result<TimeDelta, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(_) => TimeDelta::zero(),
                Countable::Phase(p) => p.lock()?.time,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Set Countable Time Checked`

        Since this function does not recurse
        it will only change elements that hold a time value themselves not any descendants

        # Arguments
          * `countable`: &[CountableId]
          * `time`: [TimeDelta]; The new time for the `Countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn set_time(&self, countable: &CountableId, time: TimeDelta) -> Result<(), AppError> {
        match self.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(_) => (),
            Countable::Phase(p) => p.lock()?.time = time,
            Countable::Chain(_) => todo!(),
        };

        self.is_changed.replace(true);

        Ok(())
    }

    /**
        `Add Time Checked`

        Since this function does not recurse
        it will only change elements that hold a count value themselves not any descendants

        # Arguments
          * `countable`: &[CountableId]
          * `time`: [TimeDelta]; The time to add to `countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn add_time(&self, countable: &CountableId, time: TimeDelta) -> Result<(), AppError> {
        match self.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(_) => (),
            Countable::Phase(p) => p.lock()?.time += time,
            Countable::Chain(_) => todo!(),
        };

        self.is_changed.replace(true);

        Ok(())
    }

    /**
        `Countable Rolls Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(i32)`: The rolls of the `Countable` for the given `CountableId`
          * `Ok(0)`: The rolls of `countable` is dependant on descendants use recursive instead
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn rolls(&self, countable: &CountableId) -> Result<i32, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(_) => 0,
                Countable::Phase(_) => self.recursive_ref().hunttype(countable)?.rolls()(
                    self.count(countable)?,
                    self.has_charm_checked(countable)?,
                ),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Countable Odds Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(f64)`: The odds of the `Countable` for the given `CountableId`
          * `Ok(0.0)`: The odds of `countable` is dependant on descendants use recursive instead
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn odds(&self, countable: &CountableId) -> Result<f64, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(_) => 0.0,
                Countable::Phase(p) => p.lock()?.hunt_type.odds(),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Countable Progress Checked`

        This function will calculate the progress on a given `countable`,
        this means the percentage chance you have to be already done with the hunt.

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(f64)`: The odds of the `Countable` for the given `CountableId`
          * `Ok(0.0)`: The odds of `countable` is dependant on descendants use recursive instead
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn progress(&self, countable: &CountableId) -> Result<f64, AppError> {
        let prob = 1.0 / self.odds(countable)?;
        let rolls = self.rolls(countable)?;
        Ok(1.0 - (1.0 - prob).powi(rolls))
    }

    /**
        `Completed Countable Checked`

        This function return a boolean on wether the countable has the completed flag toggled
        To get a number of completed descendants use the recursive version instead

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(bool)`: The completed status of the `Countable` for the given `CountableId`
          * `Ok(false)`: The completed status of `countable` is dependant on descendants use recursive instead
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn completed(&self, countable: &CountableId) -> Result<bool, AppError> {
        Ok(
            match self.get(countable).ok_or(AppError::CountableNotFound)? {
                Countable::Counter(_) => false,
                Countable::Phase(p) => p.lock()?.success,
                Countable::Chain(_) => todo!(),
            },
        )
    }
}

impl CountableStore<Recursive, Checked> {
    /**
        `Recursive Children Checked`

        Returns this list of all `descendants`.
        To check for direct children use the `Level` version of this function.

        This function will return an `error` when `CountableId` is not available in the store
        And will return an `error` when any lock on a `Mutex` fails

        `descendant`: as opposed to `direct` children they include even children of children

        [CountableId]
    */
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

    /**
        `Recursive Has Child Checked`

        Returns `true` if any component down the tree contains the provided `CountableId`.
        To check for direct children use the `Level` version of this function.

        This function will return an `error` when `CountableId` is not available in the store
        And will return an `error` when any lock on a `Mutex` fails

        [CountableId]
    */
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
        `Root Parent Checked`

        Returns the parent `root element` of the given `CountableId`
        When the element is a root node it returns the `CountableId` of the element itself

        This function will return an `error` when `CountableId` is not available in the store
        And will return an `error` when any lock on a `Mutex` fails

        `root element`: An element without a parent

        [CountableId]
    */
    pub fn root_parent(&self, countable: &CountableId) -> Result<CountableId, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    if let Some(parent) = c.lock()?.parent {
                        self.root_parent(&parent)?
                    } else {
                        *countable
                    }
                }
                Countable::Phase(p) => self.root_parent(&p.lock()?.parent)?,
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
            store: &'a CountableStore<Recursive, Checked>,
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

    /**
        `Recursive Countable Count Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(i32)`: The count of the `countable` for the given `CountableId`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]
    */
    pub fn count(&self, countable: &CountableId) -> Result<i32, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    let mut sum = 0;
                    for child in c.lock()?.children.iter() {
                        sum += self.count(child)?;
                    }
                    sum
                }
                Countable::Phase(p) => p.lock()?.count,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Recursive Set Count Checked`

        When setting count for a countable in recursive mode the following steps apply in order:
            1. Calculate the difference between the old and new count
            2. If the given countable holds its own value add the difference if the value goes below 0 set it to zero and move to the next step
            3. Add the difference to the last child of the given countable
            4. If the above results in the count of the last child to go below 0 set it to zero instead and move on to the second to last child with the remainder
            5. repeat the above until either the count is correctly set when summing all descendants or when all descendants are 0

        # Arguments
          * `countable`: &[CountableId]
          * `count`: i32; The new count for the `Countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn set_count(&self, countable: &CountableId, count: i32) -> Result<(), AppError> {
        let mut diff = count - self.count(countable)?;

        match self.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(_) => {
                let children = self.children(countable)?;
                for child in children.into_iter().rev() {
                    diff += self.count(&child)?;
                    if diff < 0 {
                        self.set_count(&child, 0)?
                    } else {
                        self.set_count(&child, diff)?;
                        break;
                    }
                }
            }
            Countable::Phase(p) => p.lock()?.count += diff,
            Countable::Chain(_) => todo!(),
        };

        self.is_changed.replace(true);

        Ok(())
    }

    /**
        `Recursive Add Count Checked`

        When setting count for a countable in recursive mode the following steps apply in order:
            1. Calculate the difference between the old and new count
            2. If the given countable holds its own value add the difference if the value goes below 0 set it to zero and move to the next step
            3. Add the difference to the last child of the given countable
            4. If the above results in the count of the last child to go below 0 set it to zero instead and move on to the second to last child with the remainder
            5. repeat the above until either the count is correctly set when summing all descendants or when all descendants are 0

        # Arguments
          * `countable`: &[CountableId]
          * `count`: i32; The count to add to `Countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn add_count(&self, countable: &CountableId, count: i32) -> Result<(), AppError> {
        let mut diff = count;

        match self.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(_) => {
                let children = self.children(countable)?;
                for child in children.into_iter().rev() {
                    diff += self.count(&child)?;
                    if diff < 0 {
                        self.set_count(&child, 0)?
                    } else {
                        self.set_count(&child, diff)?;
                        break;
                    }
                }
            }
            Countable::Phase(p) => p.lock()?.count += diff,
            Countable::Chain(_) => todo!(),
        };

        self.is_changed.replace(true);

        Ok(())
    }

    /**
        `Recursive Countable Time Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(TimeDelta)`: The time of the `Countable` for the given `CountableId`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn time(&self, countable: &CountableId) -> Result<TimeDelta, AppError> {
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(c) => {
                    let mut sum = TimeDelta::zero();
                    for child in c.lock()?.children.iter() {
                        sum += self.time(child)?;
                    }
                    sum
                }
                Countable::Phase(p) => p.lock()?.time,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Recursive Set Time Checked`

        When setting time for a countable in recursive mode the following steps apply in order:
            1. Calculate the difference between the old and new time
            2. If the given countable holds its own value add the difference if the value goes below 0 set it to zero and move to the next step
            3. Add the difference to the last child of the given countable
            4. If the above results in the time of the last child going below 0 set it to zero instead and move on to the second to last child with the remainder
            5. repeat the above until either the time is correctly set when summing all descendants or when all descendants are 0

        # Arguments
          * `countable`: &[CountableId]
          * `time`: [TimeDelta]; The new time for the `Countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn set_time(&self, countable: &CountableId, time: TimeDelta) -> Result<(), AppError> {
        let mut diff = time - self.time(countable)?;

        match self.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(_) => {
                let children = self.children(countable)?;
                for child in children.into_iter().rev() {
                    diff += self.time(&child)?;
                    if diff < TimeDelta::zero() {
                        self.set_time(&child, TimeDelta::zero())?
                    } else {
                        self.set_time(&child, diff)?;
                        break;
                    }
                }
            }
            Countable::Phase(p) => p.lock()?.time += diff,
            Countable::Chain(_) => todo!(),
        };

        self.is_changed.replace(true);

        Ok(())
    }

    /**
        `Recursive Add Time Checked`

        When setting time for a countable in recursive mode the following steps apply in order:
            1. Calculate the difference between the old and new time
            2. If the given countable holds its own value add the difference if the value goes below 0 set it to zero and move to the next step
            3. Add the difference to the last child of the given countable
            4. If the above results in the time of the last child to go below 0 set it to zero instead and move on to the second to last child with the remainder
            5. repeat the above until either the time is correctly set when summing all descendants or when all descendants are 0

        # Arguments
          * `countable`: &[CountableId]
          * `time`: [TimeDelta]; The time to add to `Countable`

        # Returns
          * `Ok(())`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [AppError]
    */
    pub fn add_time(&self, countable: &CountableId, time: TimeDelta) -> Result<(), AppError> {
        let mut diff = time;

        match self.get(countable).ok_or(AppError::CountableNotFound)? {
            Countable::Counter(_) => {
                let children = self.children(countable)?;
                for child in children.into_iter().rev() {
                    diff += self.time(&child)?;
                    if diff < TimeDelta::zero() {
                        self.set_time(&child, TimeDelta::zero())?
                    } else {
                        self.set_time(&child, diff)?;
                        break;
                    }
                }
            }
            Countable::Phase(p) => p.lock()?.time += diff,
            Countable::Chain(_) => todo!(),
        };

        self.is_changed.replace(true);

        Ok(())
    }

    /**
        `Countable Hunttype Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(Hunttype)`: The hunttype of the `countable` for the given `CountableId`
          * `Ok(Hunttype::Mixed)`: When children have differing hunttypes
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [Hunttype]\
        [AppError]
    */
    pub fn hunttype(&self, countable: &CountableId) -> Result<Hunttype, AppError> {
        Ok(
            match self.get(countable).ok_or(AppError::CountableNotFound)? {
                Countable::Counter(_) => {
                    let mut hunttype: Option<Hunttype> = None;
                    for child in self.children(countable)? {
                        if let Some(ht) = hunttype {
                            hunttype = Some(ht | self.hunttype(&child)?)
                        } else {
                            hunttype = Some(self.hunttype(&child)?)
                        };
                    }
                    hunttype.ok_or(AppError::RequiresChild)?
                }
                Countable::Phase(p) => p.lock()?.hunt_type,
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Recursive Countable Rolls Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(i32)`: The time of the `Countable` for the given `CountableId`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn rolls(&self, countable: &CountableId) -> Result<i32, AppError> {
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
                    .map(|child| self.rolls(child))
                    .collect::<Result<Vec<_>, AppError>>()?
                    .into_iter()
                    .sum(),
                Countable::Phase(_) => self.hunttype(countable)?.rolls()(
                    self.count(countable)?,
                    self.has_charm_checked(countable)?,
                ),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Recursive Countable Odds Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(f64)`: The odds of the `Countable` for the given `CountableId`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn odds(&self, countable: &CountableId) -> Result<f64, AppError> {
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
                            let odds = self.odds(child)?;
                            Ok(odds * self.count(child)? as f64)
                        })
                        .collect::<Result<Vec<_>, AppError>>()?
                        .into_iter()
                        .sum::<f64>();
                    sum / (self.count(countable)? as f64).max(1.0)
                }
                Countable::Phase(p) => p.lock()?.hunt_type.odds(),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Recursive Countable Progress Checked`

        This function will calculate the progress on a given `countable`,
        this means the percentage chance you have to be already done with the hunt.

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(f64)`: The odds of the `Countable` for the given `CountableId`
          * `Ok(0.0)`: The odds of `countable` is dependant on descendants use recursive instead
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn progress(&self, countable: &CountableId) -> Result<f64, AppError> {
        let prob = 1.0 / self.odds(countable)?;
        let rolls = self.rolls(countable)?;
        Ok(
            match self
                .store
                .get(countable)
                .ok_or(AppError::CountableNotFound)?
            {
                Countable::Counter(_) => {
                    let children_len = self.children(countable)?.len();
                    let mut chance = 0.0;

                    for k in 0..=((self.completed(countable)? as usize).min(children_len)) {
                        let combs = if rolls >= 0 {
                            n_choose_k(rolls as usize, k)
                        } else {
                            0.0
                        };
                        chance += combs * prob.powi(k as i32) * (1.0 - prob).powi(rolls - k as i32)
                    }

                    1.0 - chance
                }
                Countable::Phase(_) => 1.0 - (1.0 - prob).powi(rolls),
                Countable::Chain(_) => todo!(),
            },
        )
    }

    /**
        `Recursive Countable Completed Checked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Ok(i32)`: The amount of completed descendants of `countable`
          * `Err(AppError)`

        # Errors
          * [AppError::CountableNotFound]
          * [AppError::LockMutex]

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn completed(&self, countable: &CountableId) -> Result<u32, AppError> {
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
                    .map(|child| self.completed(child))
                    .collect::<Result<Vec<_>, AppError>>()?
                    .into_iter()
                    .sum::<u32>(),
                Countable::Phase(p) => p.lock()?.success.into(),
                Countable::Chain(_) => todo!(),
            },
        )
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

    /**
        `Countable Count UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `i32` | The count of the `countable` for the given `CountableId`
          * `0`   | The `countable` was not found in `CountableStore`

        # Panics
          * lock on a `Mutex` fails
    */
    pub fn count(&self, countable: &CountableId) -> i32 {
        match self.checked_ref().count(countable) {
            Ok(num) => num,
            Err(AppError::CountableNotFound) => 0,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Set Countable Count UnChecked`

        Since this function does not recurse
        it will only change elements that hold a count value themselves not any descendants

        # Arguments
          * `countable`: &[CountableId]
          * `count`: i32 | The new count for the `countable`

        # Panics
          * lock on a `Mutex` fails
    */
    pub fn set_count(&self, countable: &CountableId, count: i32) {
        match self.checked_ref().set_count(countable, count) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Add Count UnChecked`

        Since this function does not recurse
        it will only change elements that hold a count value themselves not any descendants

        # Arguments
          * `countable`: &[CountableId]
          * `count`: i32 | The count to add to `countable`

        # Panics
          * lock on a `Mutex` fails
    */
    pub fn add_count(&self, countable: &CountableId, count: i32) {
        match self.checked_ref().add_count(countable, count) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Countable Time UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `TimeDelta`: The time of the `countable` for the given `CountableId`
          * `TimeDelta::zero()`: `countable` was not found in `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        [TimeDelta]
    */
    pub fn time(&self, countable: &CountableId) -> TimeDelta {
        match self.checked_ref().time(countable) {
            Ok(time) => time,
            Err(AppError::CountableNotFound) => TimeDelta::zero(),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Set Countable Time UnChecked`

        Since this function does not recurse
        it will only change elements that hold a time value themselves not any descendants

        # Arguments
          * `countable`: &[CountableId]
          * `time`: [TimeDelta]; The new time for the `Countable`

        # Panics
          * lock on a `Mutex` fails
    */
    pub fn set_time(&self, countable: &CountableId, time: TimeDelta) {
        match self.checked_ref().set_time(countable, time) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Add Time UnChecked`

        Since this function does not recurse
        it will only change elements that hold a count value themselves not any descendants

        # Arguments
          * `countable`: &[CountableId]
          * `time`: [TimeDelta]; The time to add to `countable`

        # Panics
          * lock on a `Mutex` fails
    */
    pub fn add_time(&self, countable: &CountableId, time: TimeDelta) {
        match self.checked_ref().add_time(countable, time) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Countable Rolls UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `i32`: The rolls of the `Countable` for the given `CountableId`
          * `0`: The `countable` was not available in `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        [TimeDelta]
    */
    pub fn rolls(&self, countable: &CountableId) -> i32 {
        match self.checked_ref().rolls(countable) {
            Ok(r) => r,
            Err(AppError::CountableNotFound) | Err(AppError::RequiresChild) => 0,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Countable Odds UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `i32`: The odds of the `Countable` for the given `CountableId`
          * `0`: The `countable` was not available in `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        [TimeDelta]
    */
    pub fn odds(&self, countable: &CountableId) -> f64 {
        match self.checked_ref().odds(countable) {
            Ok(r) => r,
            Err(AppError::CountableNotFound) | Err(AppError::RequiresChild) => 0.0,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Countable Progress UnChecked`

        This function will calculate the progress on a given `countable`,
        this means the percentage chance you have to be already done with the hunt.

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `f64`: The odds of the `Countable` for the given `CountableId`
          * `0.0`: The odds of `countable` is dependant on descendants use recursive instead
                   or the `countable` was not available in `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        [TimeDelta]
    */
    pub fn progress(&self, countable: &CountableId) -> f64 {
        match self.checked_ref().progress(countable) {
            Ok(p) => p,
            Err(AppError::CountableNotFound) | Err(AppError::RequiresChild) => 0.0,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Countable Completed UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `bool`: Returns a whether the `countable` has the completed status
          * `false`: The `countable` was not found in `CountableStore` or
                     the completed status is dependant on its descendants

        # Panics
          * lock on a `Mutex` fails

        [TimeDelta]
    */
    pub fn completed(&self, countable: &CountableId) -> bool {
        match self.checked_ref().completed(countable) {
            Ok(b) => b,
            Err(AppError::CountableNotFound) => false,
            Err(err) => panic!("{err}"),
        }
    }
}

impl CountableStore<Recursive, UnChecked> {
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
        match self.checked_ref().root_parent(countable) {
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

    /**
        `Recursive Countable Count UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `i32`: The count of the `Countable` for the given `CountableId`
          * `0`: The `CountableId` was not in `CounterStore`
          * `Err(AppError)`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [AppError]
    */
    pub fn count(&self, countable: &CountableId) -> i32 {
        match self.checked_ref().count(countable) {
            Ok(num) => num,
            Err(AppError::CountableNotFound) => 0,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Set Count UnChecked`

        When setting count for a countable in recursive mode the following steps apply in order:
            1. Calculate the difference between the old and new count
            2. If the given countable holds its own value add the difference if the value goes below 0 set it to zero and move to the next step
            3. Add the difference to the last child of the given countable
            4. If the above results in the count of the last child to go below 0 set it to zero instead and move on to the second to last child with the remainder
            5. repeat the above until either the count is correctly set when summing all descendants or when all descendants are 0

        # Arguments
          * `countable`: &[CountableId]
          * `count`: i32; The new count for the `Countable`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [AppError]
    */
    pub fn set_count(&self, countable: &CountableId, count: i32) {
        match self.checked_ref().set_count(countable, count) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Add Count Checked`

        When setting count for a countable in recursive mode the following steps apply in order:
            1. Calculate the difference between the old and new count
            2. If the given countable holds its own value add the difference if the value goes below 0 set it to zero and move to the next step
            3. Add the difference to the last child of the given countable
            4. If the above results in the count of the last child to go below 0 set it to zero instead and move on to the second to last child with the remainder
            5. repeat the above until either the count is correctly set when summing all descendants or when all descendants are 0

        # Arguments
          * `countable`: &[CountableId]
          * `count`: i32; The count to add to `Countable`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [AppError]
    */
    pub fn add_count(&self, countable: &CountableId, count: i32) {
        match self.checked_ref().add_count(countable, count) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Countable Time UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `TimeDelta`: The time of the `Countable` for the given `CountableId`
          * `TimeDelta::zero()`: The `CountableId` was not found in `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn time(&self, countable: &CountableId) -> TimeDelta {
        match self.checked_ref().time(countable) {
            Ok(time) => time,
            Err(AppError::CountableNotFound) => TimeDelta::zero(),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Set Time UnChecked`

        When setting time for a countable in recursive mode the following steps apply in order:
            1. Calculate the difference between the old and new time
            2. If the given countable holds its own value add the difference if the value goes below 0 set it to zero and move to the next step
            3. Add the difference to the last child of the given countable
            4. If the above results in the time of the last child going below 0 set it to zero instead and move on to the second to last child with the remainder
            5. repeat the above until either the time is correctly set when summing all descendants or when all descendants are 0

        # Arguments
          * `countable`: &[CountableId]
          * `time`: [TimeDelta]; The new time for the `Countable`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [AppError]
    */
    pub fn set_time(&self, countable: &CountableId, time: TimeDelta) {
        match self.checked_ref().set_time(countable, time) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Add Time UnChecked`

        When setting time for a countable in recursive mode the following steps apply in order:
            1. Calculate the difference between the old and new time
            2. If the given countable holds its own value add the difference if the value goes below 0 set it to zero and move to the next step
            3. Add the difference to the last child of the given countable
            4. If the above results in the time of the last child to go below 0 set it to zero instead and move on to the second to last child with the remainder
            5. repeat the above until either the time is correctly set when summing all descendants or when all descendants are 0

        # Arguments
          * `countable`: &[CountableId]
          * `time`: [TimeDelta]; The time to add to `Countable`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [AppError]
    */
    pub fn add_time(&self, countable: &CountableId, time: TimeDelta) {
        match self.checked_ref().add_time(countable, time) {
            Ok(_) | Err(AppError::CountableNotFound) => (),
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Countable Hunttype UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `Hunttype`: The hunttype of the `countable` for the given `CountableId`
          * `Hunttype::Mixed`: When descendants have differing hunttypes

        # Panics
          * `countable` or any of its descendants not available in the store
          * lock on a `Mutex` fails

        [Countable]\
        [Hunttype]\
        [AppError]
    */
    pub fn hunttype(&self, countable: &CountableId) -> Hunttype {
        match self.checked_ref().hunttype(countable) {
            Ok(t) => t,
            Err(AppError::CountableNotFound) | Err(AppError::RequiresChild) => Hunttype::Mixed,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Countable Rolls UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `i32`: The time of the `Countable` for the given `CountableId`
          * 0: The `countable` was not available in `CountableStore`

        # Panics
          * `CountableId` is not available in the store
          * lock on a `Mutex` fails

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn rolls(&self, countable: &CountableId) -> i32 {
        match self.checked_ref().rolls(countable) {
            Ok(r) => r,
            Err(AppError::CountableNotFound) | Err(AppError::RequiresChild) => 0,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Countable Odds UnChecked`

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `i32`: The odds of the `Countable` for the given `CountableId`
          * `0`: The `countable` was not available in `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn odds(&self, countable: &CountableId) -> f64 {
        match self.checked_ref().odds(countable) {
            Ok(r) => r,
            Err(AppError::CountableNotFound) | Err(AppError::RequiresChild) => 0.0,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Countable Progress UnChecked`

        This function will calculate the progress on a given `countable`,
        this means the percentage chance you have to be already done with the hunt.

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `f64`: The odds of the `Countable` for the given `CountableId`
          * `0.0`: The `countable` was not available in `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn progress(&self, countable: &CountableId) -> f64 {
        match self.checked_ref().progress(countable) {
            Ok(p) => p,
            Err(AppError::CountableNotFound) | Err(AppError::RequiresChild) => 0.0,
            Err(err) => panic!("{err}"),
        }
    }

    /**
        `Recursive Completed Countable Checked`

        This function return the total number of descendants with the completed flag set

        # Arguments
          * `countable`: &[CountableId]

        # Returns
          * `bool`: The completed status of the `Countable` for the given `CountableId`
          * `false`: The `countable` was not available in `CountableStore`

        # Panics
          * lock on a `Mutex` fails

        [Countable]\
        [TimeDelta]\
        [AppError]
    */
    pub fn completed(&self, countable: &CountableId) -> u32 {
        match self.checked_ref().completed(countable) {
            Ok(c) => c,
            Err(AppError::CountableNotFound) => 0,
            Err(err) => panic!("{err}"),
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
