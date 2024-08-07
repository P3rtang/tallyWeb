#![allow(clippy::assign_op_pattern)]
use chrono::Duration;
use components::SelectOption;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    any::Any,
    cmp::Ordering,
    sync::{Arc, Mutex},
};

use super::*;

#[derive(Debug, Clone)]
pub struct ArcCountable(Arc<Mutex<Box<dyn Countable>>>);

impl ArcCountable {
    pub fn new(countable: Box<dyn Countable>) -> Self {
        Self(Arc::new(Mutex::new(countable)))
    }

    pub fn kind(&self) -> CountableKind {
        self.0
            .try_lock()
            .map(|c| c.kind())
            .unwrap_or(CountableKind::Counter)
    }

    pub fn is_active(&self) -> bool {
        self.0.try_lock().map(|c| c.is_active()).unwrap_or_default()
    }

    pub fn set_active(&self, set: bool) {
        let _ = self.0.try_lock().map(|mut c| c.set_active(set));
    }

    pub fn get_uuid(&self) -> uuid::Uuid {
        self.0.try_lock().map(|c| c.get_uuid()).unwrap_or_default()
    }

    pub fn get_name(&self) -> String {
        self.0.try_lock().map(|c| c.get_name()).unwrap_or_default()
    }

    pub fn set_name(&self, name: String) {
        let _ = self.0.try_lock().map(|mut c| c.set_name(name));
    }

    pub fn get_count(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_count()).unwrap_or_default()
    }

    pub fn set_count(&self, count: i32) {
        let _ = self.0.try_lock().map(|mut c| c.set_count(count));
    }

    pub fn add_count(&self, count: i32) {
        let _ = self.0.try_lock().map(|mut c| c.add_count(count));
    }

    pub fn get_rolls(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_rolls()).unwrap_or_default()
    }

    pub fn get_odds(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_odds()).unwrap_or(8192)
    }

    pub fn get_time(&self) -> Duration {
        self.0
            .try_lock()
            .map(|c| c.get_time())
            .unwrap_or(Duration::zero())
    }

    pub fn add_time(&self, dur: Duration) {
        let _ = self.0.try_lock().map(|mut c| c.add_time(dur));
    }

    pub fn set_time(&self, dur: Duration) {
        let _ = self.0.try_lock().map(|mut c| c.set_time(dur));
    }

    pub fn get_progress(&self) -> f64 {
        self.0
            .try_lock()
            .map(|c| c.get_progress())
            .unwrap_or_default()
    }

    pub fn get_hunt_type(&self) -> Hunttype {
        self.0
            .try_lock()
            .map(|c| c.get_hunt_type())
            .unwrap_or_default()
    }

    pub fn set_hunt_type(&self, hunt_type: Hunttype) {
        let _ = self.0.try_lock().map(|mut c| c.set_hunt_type(hunt_type));
    }

    pub fn has_charm(&self) -> bool {
        self.0.try_lock().map(|c| c.has_charm()).unwrap_or_default()
    }

    pub fn set_charm(&self, set: bool) {
        let _ = self.0.try_lock().map(|mut c| c.set_charm(set));
    }

    pub fn new_phase(&self, name: String) -> ArcCountable {
        let _ = self.0.try_lock().map(|mut c| c.new_phase(name));
        self.get_children().last().cloned().unwrap()
    }

    pub fn get_children(&self) -> Vec<ArcCountable> {
        self.0
            .try_lock()
            .map_or_else(
                |_| Vec::new(),
                |c| c.get_phases().into_iter().cloned().collect(),
            )
            .clone()
    }

    pub fn created_at(&self) -> chrono::NaiveDateTime {
        self.0
            .try_lock()
            .map(|c| c.created_at())
            .unwrap_or_default()
    }

    pub fn has_child_starts_with(&self, pat: &str) -> bool {
        let mut has_child = self.get_name().to_lowercase().starts_with(pat);
        for child in self.get_children() {
            has_child |= child.has_child_starts_with(pat)
        }
        has_child
    }

    pub fn has_child_contains(&self, pat: &str) -> bool {
        let mut contains = self.get_name().to_lowercase().contains(pat);
        for child in self.get_children() {
            contains |= child.has_child_contains(pat)
        }
        contains
    }

    pub fn as_any(&self) -> Result<Box<dyn core::any::Any + 'static>, AppError> {
        let c = self.0.try_lock().map_err(|_| AppError::LockMutex)?;
        Ok(c.box_any())
    }

    pub fn get_completed(&self) -> usize {
        self.try_lock()
            .map(|c| c.get_completed())
            .unwrap_or_default()
    }

    pub fn toggle_success(&self) {
        let _ = self.try_lock().map(|mut c| c.toggle_success());
    }
}

impl PartialEq for ArcCountable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.0, &*other.0)
    }
}

impl std::ops::Deref for ArcCountable {
    type Target = Mutex<Box<dyn Countable>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryInto<api::Countable> for ArcCountable {
    type Error = AppError;

    fn try_into(self) -> Result<api::Countable, Self::Error> {
        match self.kind() {
            CountableKind::Counter => Ok(api::Countable::Counter(
                self.as_any()?
                    .downcast_ref()
                    .cloned()
                    .ok_or(AppError::AnyConversion)?,
            )),
            CountableKind::Phase => Ok(api::Countable::Phase(
                self.as_any()?
                    .downcast_ref()
                    .cloned()
                    .ok_or(AppError::AnyConversion)?,
            )),
        }
    }
}

impl Default for ArcCountable {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Box::<Counter>::default())))
    }
}

impl saving::Savable for leptos::RwSignal<ArcCountable> {
    fn endpoint() -> leptos::Action<(UserSession, Vec<Self>), Result<(), AppError>> {
        use leptos::SignalGetUntracked;
        leptos::create_action(|(user, countables): &(UserSession, Vec<Self>)| {
            save_countables(
                user.clone(),
                countables.iter().map(|s| (*s).get_untracked()).collect(),
            )
        })
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Hunttype {
    #[default]
    OldOdds,
    NewOdds,
    SOS,
    DexNav(i32),
    Masuda(Masuda),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Masuda {
    GenIV,
    GenV,
    #[default]
    GenVI,
}

impl Hunttype {
    fn get_rolls(&self, count: i32, has_charm: bool) -> i32 {
        match self {
            Hunttype::OldOdds | Hunttype::NewOdds => {
                if has_charm {
                    count * 3
                } else {
                    count
                }
            }
            Hunttype::SOS => {
                let mut rolls = if count < 10 {
                    count
                } else if count < 20 {
                    10 + (count - 10) * 5
                } else if count < 30 {
                    60 + (count - 20) * 9
                } else {
                    150 + (count - 30) * 13
                };

                if has_charm {
                    rolls += count * 2
                }
                rolls
            }
            Hunttype::DexNav(_) => todo!(),
            Hunttype::Masuda(Masuda::GenIV) => {
                if has_charm {
                    count * 7
                } else {
                    count * 5
                }
            }
            Hunttype::Masuda(_) => {
                if has_charm {
                    count * 8
                } else {
                    count * 6
                }
            }
        }
    }

    fn get_odds(&self) -> i32 {
        match self {
            Hunttype::OldOdds => 8192,
            Hunttype::Masuda(Masuda::GenIV | Masuda::GenV) => 8192,
            _ => 4096,
        }
    }

    pub fn repr(&self) -> &'static str {
        match self {
            Hunttype::OldOdds => "Old Odds",
            Hunttype::NewOdds => "New Odds",
            Hunttype::SOS => "SOS",
            Hunttype::DexNav(_) => "DexNav",
            Hunttype::Masuda(Masuda::GenIV) => "Masuda (gen IV)",
            Hunttype::Masuda(Masuda::GenV) => "Masuda (gen V)",
            Hunttype::Masuda(Masuda::GenVI) => "Masuda (gen VI+)",
        }
    }
}

impl From<Hunttype> for &'static str {
    fn from(val: Hunttype) -> Self {
        match val {
            Hunttype::OldOdds => "OldOdds",
            Hunttype::NewOdds => "NewOdds",
            Hunttype::SOS => "SOS",
            Hunttype::DexNav(_) => "DexNav",
            Hunttype::Masuda(Masuda::GenIV) => "MasudaGenIV",
            Hunttype::Masuda(Masuda::GenV) => "MasudaGenV",
            Hunttype::Masuda(Masuda::GenVI) => "MasudaGenVI",
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
            "DexNav" => Ok(Self::DexNav(0)),
            "MasudaGenIV" => Ok(Self::Masuda(Masuda::GenIV)),
            "MasudaGenV" => Ok(Self::Masuda(Masuda::GenV)),
            "MasudaGenVI" => Ok(Self::Masuda(Masuda::GenVI)),
            _ => Err(String::from(
                "Hunttype should be one of the following: OldOdds, NewOdds, SOS, Masuda",
            )),
        };
    }
}

impl From<Hunttype> for SelectOption {
    fn from(val: Hunttype) -> Self {
        (val.repr(), val.clone().into()).into()
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

pub trait Countable: std::fmt::Debug + Send + Any {
    fn get_uuid(&self) -> uuid::Uuid;
    fn kind(&self) -> CountableKind;

    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);

    fn get_count(&self) -> i32;
    fn set_count(&mut self, count: i32);
    fn add_count(&mut self, count: i32);

    fn get_rolls(&self) -> i32;
    fn get_odds(&self) -> i32;

    fn get_time(&self) -> Duration;
    fn set_time(&mut self, dur: Duration);
    fn add_time(&mut self, dur: Duration);

    fn is_active(&self) -> bool;
    fn toggle_active(&mut self);
    fn set_active(&mut self, active: bool);

    fn get_progress(&self) -> f64;
    fn get_hunt_type(&self) -> Hunttype;
    fn set_hunt_type(&mut self, hunt_type: Hunttype);
    fn has_charm(&self) -> bool;
    fn set_charm(&mut self, set: bool);
    fn get_completed(&self) -> usize;
    fn toggle_success(&mut self);

    fn created_at(&self) -> chrono::NaiveDateTime;

    fn new_phase(&mut self, name: String);
    fn new_counter(&mut self, name: String, owner: uuid::Uuid) -> Option<ArcCountable>;

    fn get_phases(&self) -> Vec<&ArcCountable>;
    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable>;

    fn has_children(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn box_any(&self) -> Box<dyn Any>;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SerCounter {
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub name: String,
    pub phase_list: Vec<Phase>,
    pub created_at: chrono::NaiveDateTime,
}

impl From<Counter> for SerCounter {
    fn from(value: Counter) -> Self {
        let mut phase_list = Vec::new();
        for arc_p in value.phase_list {
            if let Some(phase) = arc_p
                .lock()
                .map(|c| c.as_any().downcast_ref::<Phase>().cloned())
                .ok()
                .flatten()
            {
                phase_list.push(phase.clone())
            }
        }
        SerCounter {
            uuid: value.uuid,
            owner_uuid: value.owner_uuid,
            name: value.name,
            phase_list,
            created_at: value.created_at,
        }
    }
}

impl Countable for SerCounter {
    fn get_uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    fn kind(&self) -> CountableKind {
        CountableKind::Counter
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name
    }

    fn get_count(&self) -> i32 {
        self.phase_list.iter().map(|p| p.get_count()).sum()
    }

    fn set_count(&mut self, count: i32) {
        let mut diff = self.get_count() - count;
        for phase in self.phase_list.iter_mut().rev() {
            if phase.get_count() < diff {
                diff -= phase.get_count();
                phase.set_count(0);
            } else {
                phase.set_count(phase.get_count() - diff);
                break;
            }
        }
    }

    fn add_count(&mut self, count: i32) {
        let mut diff = count;
        for phase in self.phase_list.iter_mut().rev() {
            if phase.get_count() < diff {
                diff -= phase.get_count();
                phase.set_count(0);
            } else {
                phase.set_count(phase.get_count() - diff);
                break;
            }
        }
    }

    fn get_rolls(&self) -> i32 {
        self.phase_list.iter().map(|p| p.get_rolls()).sum()
    }

    fn get_odds(&self) -> i32 {
        self.phase_list
            .last()
            .map(|p| p.get_rolls())
            .unwrap_or(8192)
    }

    fn get_time(&self) -> Duration {
        self.phase_list.iter().map(|p| p.get_time()).sum()
    }

    fn set_time(&mut self, dur: Duration) {
        let mut diff = self.get_time() - dur;
        for phase in self.phase_list.iter_mut().rev() {
            if phase.get_time() < diff {
                diff = diff - phase.get_time();
                phase.set_time(Duration::zero());
            } else {
                phase.set_time(phase.get_time() - diff);
                break;
            }
        }
    }

    fn add_time(&mut self, dur: Duration) {
        if let Some(p) = self.phase_list.last_mut() {
            p.add_time(dur);
        }
    }

    fn is_active(&self) -> bool {
        todo!()
    }

    fn toggle_active(&mut self) {
        todo!()
    }

    fn set_active(&mut self, _active: bool) {
        todo!()
    }

    fn get_progress(&self) -> f64 {
        todo!()
    }

    fn get_hunt_type(&self) -> Hunttype {
        self.phase_list
            .last()
            .map(|p| p.get_hunt_type())
            .unwrap_or_default()
    }

    fn set_hunt_type(&mut self, hunt_type: Hunttype) {
        self.phase_list
            .iter_mut()
            .for_each(|p| p.set_hunt_type(hunt_type.clone()));
    }

    fn has_charm(&self) -> bool {
        self.phase_list
            .last()
            .map(|p| p.has_charm())
            .unwrap_or_default()
    }

    fn set_charm(&mut self, set: bool) {
        self.phase_list.iter_mut().for_each(|p| p.set_charm(set))
    }

    fn created_at(&self) -> chrono::NaiveDateTime {
        todo!()
    }

    fn new_phase(&mut self, _name: String) {
        todo!()
    }

    fn new_counter(&mut self, _name: String, _owner: uuid::Uuid) -> Option<ArcCountable> {
        todo!()
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        todo!()
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        todo!()
    }

    fn has_children(&self) -> bool {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn box_any(&self) -> Box<dyn Any> {
        Box::new(self.clone())
    }

    fn get_completed(&self) -> usize {
        todo!()
    }

    fn toggle_success(&mut self) {
        self.phase_list.iter_mut().for_each(|p| p.toggle_success());
    }
}

#[derive(Debug, Clone, Default)]
pub struct Counter {
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub name: String,
    pub phase_list: Vec<ArcCountable>,
    pub created_at: chrono::NaiveDateTime,
}

#[allow(dead_code)]
impl Counter {
    pub fn new(name: impl ToString, owner_uuid: uuid::Uuid) -> Self {
        Counter {
            uuid: uuid::Uuid::new_v4(),
            owner_uuid,
            name: name.to_string(),
            phase_list: Vec::new(),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl Countable for Counter {
    fn get_uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    fn kind(&self) -> CountableKind {
        CountableKind::Counter
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name
    }

    fn get_count(&self) -> i32 {
        return self.phase_list.iter().map(|p| p.get_count()).sum();
    }

    fn set_count(&mut self, count: i32) {
        let mut diff = count - self.get_count();
        self.phase_list.sort_by_key(|a| a.created_at());
        for phase in self.phase_list.iter_mut().rev() {
            if phase.get_count() + diff <= 0 {
                phase.set_count(0);
                diff += phase.get_count();
            } else {
                phase.set_count(phase.get_count() + diff);
                break;
            }
        }
    }

    fn add_count(&mut self, count: i32) {
        if let Some(p) = self.phase_list.last_mut() {
            let _ = p.0.try_lock().map(|mut p| p.add_count(count));
        }
    }

    fn get_rolls(&self) -> i32 {
        self.phase_list.iter().map(|p| p.get_rolls()).sum()
    }

    fn get_odds(&self) -> i32 {
        // TODO: look into the proper way to handle different Odds
        // averaging will approach the correct solution but likely is not fully correct
        self.phase_list.iter().map(|p| p.get_odds()).sum::<i32>() / self.phase_list.len() as i32
    }

    fn get_time(&self) -> Duration {
        return self.phase_list.iter().map(|p| p.get_time()).sum();
    }

    fn set_time(&mut self, dur: Duration) {
        let mut diff = self.get_time() - dur;
        for phase in self.phase_list.iter_mut().rev() {
            if phase.get_time() < diff {
                diff = diff - phase.get_time();
                phase.set_time(Duration::zero());
            } else {
                phase.set_time(phase.get_time() - diff);
                break;
            }
        }
    }

    fn add_time(&mut self, dur: Duration) {
        if let Some(p) = self.phase_list.last_mut() {
            let _ = p.0.try_lock().map(|mut p| p.add_time(dur));
        }
    }

    fn is_active(&self) -> bool {
        for p in self.phase_list.iter() {
            if p.try_lock().map(|p| p.is_active()).unwrap_or_default() {
                return true;
            }
        }
        false
    }

    fn toggle_active(&mut self) {
        if self.is_active() {
            self.set_active(false)
        } else {
            self.set_active(true)
        }
    }

    fn set_active(&mut self, active: bool) {
        if !active {
            self.phase_list.iter().for_each(|p| {
                let _ = p.0.lock().map(|mut p| p.set_active(false));
            });
        } else if let Some(mut p) = self.phase_list.last_mut().and_then(|p| p.try_lock().ok()) {
            p.set_active(active)
        }
    }

    fn get_progress(&self) -> f64 {
        let mut chance = 0.0;
        let p = 1.0 / self.get_odds() as f64;

        for k in 0..((self.get_completed() + 1).min(self.phase_list.len())) {
            let combs = n_choose_k(self.get_rolls() as usize, k);

            chance += combs * p.powi(k as i32) * (1.0 - p).powi(self.get_rolls() - k as i32)
        }

        1.0 - chance
    }

    fn get_hunt_type(&self) -> Hunttype {
        self.phase_list
            .last()
            .map(|p| p.get_hunt_type())
            .unwrap_or_default()
    }

    fn set_hunt_type(&mut self, hunt_type: Hunttype) {
        self.phase_list
            .iter_mut()
            .for_each(|p| p.set_hunt_type(hunt_type.clone()));
    }

    fn has_charm(&self) -> bool {
        self.phase_list
            .last()
            .map(|c| c.has_charm())
            .unwrap_or_default()
    }

    fn set_charm(&mut self, set: bool) {
        self.phase_list.iter_mut().for_each(|c| c.set_charm(set));
    }

    fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    fn new_phase(&mut self, name: String) {
        self.phase_list.push(ArcCountable::new(Box::new(Phase::new(
            name,
            self.uuid,
            self.owner_uuid,
        ))))
    }

    fn new_counter(&mut self, name: String, owner: uuid::Uuid) -> Option<ArcCountable> {
        let arc_counter = ArcCountable::new(Box::new(Counter::new(name, owner)));
        self.phase_list.push(arc_counter.clone());
        Some(arc_counter)
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        self.phase_list.iter().collect()
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        return self.phase_list.iter_mut().collect();
    }

    fn has_children(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn box_any(&self) -> Box<dyn Any> {
        Box::new(self.clone())
    }

    fn get_completed(&self) -> usize {
        self.phase_list.iter().map(|p| p.get_completed()).sum()
    }

    fn toggle_success(&mut self) {
        self.phase_list.iter().for_each(|p| p.toggle_success());
    }
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Phase {
    pub uuid: uuid::Uuid,
    pub owner_uuid: uuid::Uuid,
    pub parent_uuid: uuid::Uuid,
    pub name: String,
    pub count: i32,
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64>")]
    pub time: Duration,
    pub is_active: bool,
    pub hunt_type: Hunttype,
    pub has_charm: bool,
    pub success: bool,
    pub created_at: chrono::NaiveDateTime,
}

impl Phase {
    pub fn new(name: impl ToString, parent_uuid: uuid::Uuid, owner_uuid: uuid::Uuid) -> Self {
        Phase {
            uuid: uuid::Uuid::new_v4(),
            owner_uuid,
            parent_uuid,
            name: name.to_string(),
            count: 0,
            time: Duration::zero(),
            is_active: false,
            hunt_type: Hunttype::NewOdds,
            has_charm: false,
            success: false,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl Countable for Phase {
    fn get_uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    fn kind(&self) -> CountableKind {
        CountableKind::Phase
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name
    }

    fn get_count(&self) -> i32 {
        self.count
    }

    fn set_count(&mut self, count: i32) {
        self.count = count;
    }

    fn add_count(&mut self, count: i32) {
        self.count += count;
    }

    fn get_rolls(&self) -> i32 {
        self.hunt_type.get_rolls(self.count, self.has_charm)
    }

    fn get_odds(&self) -> i32 {
        self.hunt_type.get_odds()
    }

    fn get_time(&self) -> Duration {
        self.time
    }

    fn set_time(&mut self, dur: Duration) {
        self.time = dur
    }

    fn add_time(&mut self, dur: Duration) {
        self.time = self.time + dur
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn toggle_active(&mut self) {
        self.is_active = !self.is_active;
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    fn get_progress(&self) -> f64 {
        1.0 - (1.0 - 1.0_f64 / self.hunt_type.get_odds() as f64).powi(self.get_rolls())
    }

    fn get_hunt_type(&self) -> Hunttype {
        self.hunt_type.clone()
    }

    fn set_hunt_type(&mut self, hunt_type: Hunttype) {
        self.hunt_type = hunt_type
    }

    fn has_charm(&self) -> bool {
        self.has_charm
    }

    fn set_charm(&mut self, set: bool) {
        self.has_charm = set
    }

    fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    fn new_phase(&mut self, _: String) {}

    fn new_counter(&mut self, _: String, _: uuid::Uuid) -> Option<ArcCountable> {
        None
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        vec![]
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        vec![]
    }

    fn has_children(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn box_any(&self) -> Box<dyn Any> {
        Box::new(self.clone())
    }

    fn get_completed(&self) -> usize {
        self.success.into()
    }

    fn toggle_success(&mut self) {
        self.success = !self.success
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum CountableKind {
    #[default]
    Counter,
    Phase,
}

impl std::fmt::Display for CountableKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CountableKind::Counter => write!(f, "Counter"),
            CountableKind::Phase => write!(f, "Phase"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortCountable {
    Id(bool),
    Name(bool),
    Count(bool),
    Time(bool),
    CreatedAt(bool),
}

impl SortCountable {
    pub fn sort_by(&self) -> impl Fn(&ArcCountable, &ArcCountable) -> Ordering {
        match self {
            SortCountable::Id(false) => {
                |a: &ArcCountable, b: &ArcCountable| a.get_uuid().cmp(&b.get_uuid())
            }
            SortCountable::Id(true) => {
                |a: &ArcCountable, b: &ArcCountable| a.get_uuid().cmp(&b.get_uuid()).reverse()
            }
            SortCountable::Name(false) => {
                |a: &ArcCountable, b: &ArcCountable| a.get_name().cmp(&b.get_name())
            }
            SortCountable::Name(true) => {
                |a: &ArcCountable, b: &ArcCountable| a.get_name().cmp(&b.get_name()).reverse()
            }
            SortCountable::Count(false) => {
                |a: &ArcCountable, b: &ArcCountable| a.get_count().cmp(&b.get_count())
            }
            SortCountable::Count(true) => {
                |a: &ArcCountable, b: &ArcCountable| a.get_count().cmp(&b.get_count()).reverse()
            }
            SortCountable::Time(false) => {
                |a: &ArcCountable, b: &ArcCountable| a.get_time().cmp(&b.get_time())
            }
            SortCountable::Time(true) => {
                |a: &ArcCountable, b: &ArcCountable| a.get_time().cmp(&b.get_time()).reverse()
            }
            SortCountable::CreatedAt(false) => {
                |a: &ArcCountable, b: &ArcCountable| a.created_at().cmp(&b.created_at())
            }
            SortCountable::CreatedAt(true) => {
                |a: &ArcCountable, b: &ArcCountable| a.created_at().cmp(&b.created_at()).reverse()
            }
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Self::Id(b) => Self::Id(!b),
            Self::Name(b) => Self::Name(!b),
            Self::Count(b) => Self::Count(!b),
            Self::Time(b) => Self::Time(!b),
            Self::CreatedAt(b) => Self::CreatedAt(!b),
        }
    }

    pub fn is_reversed(&self) -> bool {
        match self {
            SortCountable::Id(b) => *b,
            SortCountable::Name(b) => *b,
            SortCountable::Count(b) => *b,
            SortCountable::Time(b) => *b,
            SortCountable::CreatedAt(b) => *b,
        }
    }

    pub fn apply(&self, mut list: Vec<ArcCountable>) -> Vec<ArcCountable> {
        list.sort_by(self.sort_by());
        list
    }
}

impl From<String> for SortCountable {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Name" => SortCountable::Name(false),
            "Count" => SortCountable::Count(false),
            "Time" => SortCountable::Time(false),
            "Id" => SortCountable::Id(false),
            "CreatedAt" => SortCountable::CreatedAt(false),
            _ => SortCountable::Id(false),
        }
    }
}

impl From<SortCountable> for &str {
    fn from(val: SortCountable) -> Self {
        match val {
            SortCountable::Id(_) => "Id",
            SortCountable::Name(_) => "Name",
            SortCountable::Count(_) => "Count",
            SortCountable::Time(_) => "Time",
            SortCountable::CreatedAt(_) => "CreatedAt",
        }
    }
}

cfg_if::cfg_if!(
    if #[cfg(feature = "ssr")] {
        use backend::{DbCounter, DbPhase};
        use leptos_actix::extract;
        use actix_web::web;
        impl SerCounter {
            pub async fn from_db(session: &UserSession, value: DbCounter) -> Result<Self, AppError> {
                let pool = extract::<web::Data<backend::PgPool>>().await.map_err(|err| AppError::Extraction(err.to_string()))?;

                let phase_list = backend::get_phases_from_parent_uuid(&pool, session.user_uuid, value.uuid).await.map_err(|err| AppError::DatabaseError(err.to_string()))?.into_iter().map(|p| Phase::from(p)).collect();

                Ok( Self {
                    uuid: value.uuid,
                    owner_uuid: value.owner_uuid,
                    name: value.name,
                    phase_list,
                    created_at: value.created_at,
                })
            }
            pub async fn to_db(&self) -> DbCounter {
                DbCounter {
                    uuid: self.uuid,
                    owner_uuid: self.owner_uuid,
                    name: self.name.clone(),
                    created_at: self.created_at,
                }
            }
        }

        impl From<DbPhase> for Phase {
            fn from(value: DbPhase) -> Self {
                Self {
                    uuid: value.uuid,
                    owner_uuid: value.owner_uuid,
                    parent_uuid: value.parent_uuid,
                    name: value.name,
                    count: value.count,
                    time: Duration::milliseconds(value.time),
                    is_active: false,
                    hunt_type: value.hunt_type.into(),
                    has_charm: value.has_charm,
                    success: value.success,
                    created_at: value.created_at,
                }
            }
        }

        impl Phase {
            pub fn to_db(self) -> DbPhase {
                DbPhase {
                    uuid: self.uuid,
                    owner_uuid: self.owner_uuid,
                    parent_uuid: self.parent_uuid,
                    name: self.name.clone(),
                    count: self.count,
                    time: self.time.num_milliseconds(),
                    hunt_type: self.hunt_type.clone().into(),
                    has_charm: self.has_charm,
                    dexnav_encounters: self.hunt_type.into(),
                    success: self.success,
                    created_at: self.created_at,
                }
            }
        }

        impl Into<backend::Hunttype> for Hunttype {
            fn into(self) -> backend::Hunttype {
                match self {
                    Self::OldOdds               => backend::Hunttype::OldOdds,
                    Self::NewOdds               => backend::Hunttype::NewOdds,
                    Self::SOS                   => backend::Hunttype::SOS,
                    Self::DexNav(_)             => backend::Hunttype::DexNav,
                    Self::Masuda(Masuda::GenIV) => backend::Hunttype::MasudaGenIV,
                    Self::Masuda(Masuda::GenV)  => backend::Hunttype::MasudaGenV,
                    Self::Masuda(Masuda::GenVI) => backend::Hunttype::MasudaGenVI,
                }
            }
        }

        impl Into<Option<i32>> for Hunttype {
            fn into(self) -> Option<i32> {
                match self {
                    Self::DexNav(num) => Some(num),
                    _ => None,
                }
            }
        }

        impl From<backend::Hunttype> for Hunttype {
            fn from(value: backend::Hunttype) -> Self {
                match value {
                    backend::Hunttype::OldOdds     => Self::OldOdds,
                    backend::Hunttype::NewOdds     => Self::NewOdds,
                    backend::Hunttype::SOS         => Self::SOS,
                    backend::Hunttype::DexNav      => Self::DexNav(0),
                    backend::Hunttype::MasudaGenIV => Self::Masuda(Masuda::GenIV),
                    backend::Hunttype::MasudaGenV  => Self::Masuda(Masuda::GenV),
                    backend::Hunttype::MasudaGenVI => Self::Masuda(Masuda::GenVI),
                }
            }
        }
    }
);

fn n_choose_k(n: usize, k: usize) -> f64 {
    match (n, k) {
        (n, k) if k > n => 0.0,
        (_, 0) => 1.0,
        (n, k) if k > n / 2 => n_choose_k(n, n - k),
        (n, k) => n as f64 / k as f64 * n_choose_k(n - 1, k - 1),
    }
}

fn insert_deep(c: ArcCountable, map: &mut HashMap<uuid::Uuid, ArcCountable>) {
    map.insert(c.get_uuid(), c.clone());
    for child in c.get_children() {
        insert_deep(child, map)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CounterList {
    pub list: HashMap<uuid::Uuid, ArcCountable>,
    pub flat: HashMap<uuid::Uuid, ArcCountable>,
    search: Option<String>,
    pub sort: SortCountable,
}

impl CounterList {
    pub fn new(counters: &[Counter]) -> Self {
        let mut list = HashMap::new();
        let mut flat = HashMap::new();
        for c in counters {
            let countable = ArcCountable::new(Box::new(c.clone()));
            list.insert(c.uuid, countable.clone());
            insert_deep(countable.clone(), &mut flat);
        }

        CounterList {
            list,
            flat,
            search: None,
            sort: SortCountable::Name(false),
        }
    }

    pub fn search(&mut self, value: &str) {
        self.search = Some(value.to_lowercase().to_string())
    }

    pub fn get_items(&mut self) -> Vec<ArcCountable> {
        self.list.values().cloned().collect()
    }

    pub fn get_filtered_list(&mut self) -> Vec<ArcCountable> {
        let mut list = self.list.values().cloned().collect::<Vec<_>>();

        list.sort_by(self.sort.sort_by());

        if let Some(search) = &self.search {
            let mut list_starts_with = Vec::new();
            let mut child_starts_with = Vec::new();
            let mut list_contains = Vec::new();
            let mut child_contains = Vec::new();

            for counter in list.iter() {
                let name = counter.get_name().to_lowercase();
                if name.starts_with(search) {
                    list_starts_with.push(counter.clone())
                } else if counter.has_child_starts_with(search) {
                    child_starts_with.push(counter.clone())
                } else if name.contains(search) {
                    list_contains.push(counter.clone())
                } else if counter.has_child_contains(search) {
                    child_contains.push(counter.clone())
                }
            }

            list_starts_with.append(&mut child_starts_with);
            list_starts_with.append(&mut list_contains);
            list_starts_with.append(&mut child_contains);

            list_starts_with
        } else {
            list
        }
    }

    pub fn load_offline(&mut self, data: Vec<SerCounter>) {
        let list: CounterList = data.into();
        self.list = list.list;
    }
}

impl From<Vec<SerCounter>> for CounterList {
    fn from(value: Vec<SerCounter>) -> Self {
        let list = value
            .into_iter()
            .map(|sc| {
                let phase_list: Vec<ArcCountable> = sc
                    .phase_list
                    .into_iter()
                    .map(|p| ArcCountable::new(Box::new(p)))
                    .collect();
                Counter {
                    uuid: sc.uuid,
                    owner_uuid: sc.owner_uuid,
                    name: sc.name,
                    phase_list,
                    created_at: sc.created_at,
                }
            })
            .collect::<Vec<_>>();
        Self::new(list.as_slice())
    }
}

impl Default for CounterList {
    fn default() -> Self {
        Self::new(&[])
    }
}

impl From<CounterList> for Vec<SerCounter> {
    fn from(val: CounterList) -> Self {
        let mut rtrn_list = Vec::new();
        for arc_c in val.list.values() {
            if let Some(counter) = arc_c
                .lock()
                .map(|c| c.as_any().downcast_ref::<Counter>().cloned())
                .ok()
                .flatten()
            {
                rtrn_list.push(counter.clone().into())
            }
        }

        rtrn_list
    }
}
