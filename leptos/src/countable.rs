use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::app::get_phase_by_id;

#[derive(Debug, Clone)]
pub struct ArcCountable(pub Arc<Mutex<Box<dyn Countable>>>);

impl ArcCountable {
    pub fn new(countable: Box<dyn Countable>) -> Self {
        Self(Arc::new(Mutex::new(countable)))
    }

    pub fn get_uuid(&self) -> String {
        self.0.try_lock().map(|c| c.get_uuid()).unwrap_or_default()
    }

    pub fn get_id(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_id()).unwrap_or_default()
    }

    pub fn get_name(&self) -> String {
        self.0.try_lock().map(|c| c.get_name()).unwrap_or_default()
    }

    pub fn get_count(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_count()).unwrap_or_default()
    }

    pub fn add_count(&self, count: i32) {
        let _ = self.0.try_lock().map(|mut c| c.add_count(count));
    }

    pub fn get_rolls(&self) -> i32 {
        self.0.try_lock().map(|c| c.get_rolls()).unwrap_or_default()
    }

    pub fn get_time(&self) -> Duration {
        self.0.try_lock().map(|c| c.get_time()).unwrap_or_default()
    }

    pub fn add_time(&self, dur: Duration) {
        let _ = self.0.try_lock().map(|mut c| c.add_time(dur));
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

    pub fn new_phase(&self, id: i32, name: String) -> ArcCountable {
        let _ = self.0.try_lock().map(|mut c| c.new_phase(id, name));
        self.get_children().last().cloned().unwrap()
    }

    pub fn get_children(&self) -> Vec<ArcCountable> {
        self.0
            .try_lock()
            .map_or_else(
                |_| Vec::new(),
                |c| c.get_phases().into_iter().map(|p| p.clone()).collect(),
            )
            .clone()
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
        return &*self.0;
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Hunttype {
    #[default]
    OldOdds,
    NewOdds,
    SOS,
    DexNav,
}

impl Hunttype {
    fn get_rolls(&self, count: i32, has_charm: bool) -> i32 {
        match self {
            Hunttype::OldOdds | Hunttype::NewOdds => {
                return if has_charm { count * 3 } else { count }
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
                return rolls;
            }
            Hunttype::DexNav => todo!(),
        }
    }

    fn get_odds(&self) -> i32 {
        return match self {
            Hunttype::OldOdds => 8192,
            _ => 4096,
        };
    }
}

impl Into<String> for Hunttype {
    fn into(self) -> String {
        match self {
            Hunttype::OldOdds => String::from("OldOdds"),
            Hunttype::NewOdds => String::from("NewOdds"),
            Hunttype::SOS => String::from("SOS"),
            Hunttype::DexNav => String::from("DexNav"),
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
            "DexNav" => Ok(Self::DexNav),
            _ => Err(String::from(
                "Hunttype should be one of the following: OldOdds, NewOdds, SOS, DexNav",
            )),
        };
    }
}

pub trait Countable: std::fmt::Debug + Send + Any {
    fn get_id(&self) -> i32;
    fn get_uuid(&self) -> String;

    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);

    fn get_count(&self) -> i32;
    fn set_count(&mut self, count: i32);
    fn add_count(&mut self, count: i32);

    fn get_rolls(&self) -> i32;

    fn get_time(&self) -> Duration;
    fn set_time(&mut self, dur: Duration);
    fn add_time(&mut self, dur: Duration);
    fn rem_time(&mut self, dur: Duration);
    fn get_progress(&self) -> f64;
    fn is_active(&self) -> bool;
    fn toggle_active(&mut self);
    fn set_active(&mut self, active: bool);

    fn get_hunt_type(&self) -> Hunttype;
    fn set_hunt_type(&mut self, hunt_type: Hunttype);
    fn has_charm(&self) -> bool;
    fn set_charm(&mut self, set: bool);

    fn new_phase(&mut self, id: i32, name: String);
    fn new_counter(&mut self, id: i32, name: String) -> Result<ArcCountable, String>;

    fn get_phases(&self) -> Vec<&ArcCountable>;
    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable>;

    fn has_children(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SerCounter {
    pub id: i32,
    pub name: String,
    pub phase_list: Vec<Phase>,
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
        return SerCounter {
            id: value.id,
            name: value.name,
            phase_list,
        };
    }
}

impl Countable for SerCounter {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_uuid(&self) -> String {
        format!("c{}", self.id)
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
        let diff = self.phase_list.iter().map(|p| p.get_count()).sum::<i32>()
            - self.phase_list.last().map(|p| p.get_count()).unwrap_or(0);
        self.phase_list
            .last_mut()
            .map(|p| p.set_count(count - diff));
    }

    fn add_count(&mut self, _count: i32) {
        todo!()
    }

    fn get_rolls(&self) -> i32 {
        todo!()
    }

    fn get_time(&self) -> Duration {
        self.phase_list.iter().map(|p| p.get_time()).sum()
    }

    fn set_time(&mut self, _dur: Duration) {
        todo!()
    }

    fn add_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            p.add_time(dur);
        });
    }

    fn rem_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            p.rem_time(dur);
        });
    }

    fn get_progress(&self) -> f64 {
        todo!()
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

    fn new_phase(&mut self, _id: i32, _name: String) {
        todo!()
    }

    fn new_counter(&mut self, _id: i32, _name: String) -> Result<ArcCountable, String> {
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
}

#[derive(Debug, Clone)]
pub struct Counter {
    pub id: i32,
    pub name: String,
    pub phase_list: Vec<ArcCountable>,
}

#[allow(dead_code)]
impl Counter {
    pub fn new(id: i32, name: impl ToString) -> Result<Self, String> {
        return Ok(Counter {
            id,
            name: name.to_string(),
            phase_list: Vec::new(),
        });
    }
}

impl Countable for Counter {
    fn get_id(&self) -> i32 {
        return self.id;
    }

    fn get_uuid(&self) -> String {
        format!("c{}", self.id)
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
        let diff = self.phase_list.iter().map(|p| p.get_count()).sum::<i32>()
            - self.phase_list.last().map(|p| p.get_count()).unwrap_or(0);
        self.phase_list
            .last_mut()
            .map(|p| p.0.try_lock().map(|mut p| p.set_count(count - diff)));
    }

    fn add_count(&mut self, count: i32) {
        self.phase_list.last_mut().map(|p| {
            let _ = p.0.try_lock().map(|mut p| p.add_count(count));
        });
    }

    fn get_rolls(&self) -> i32 {
        self.phase_list.iter().map(|p| p.get_rolls()).sum()
    }

    fn get_time(&self) -> Duration {
        return self.phase_list.iter().map(|p| p.get_time()).sum();
    }

    fn set_time(&mut self, time: Duration) {
        let diff = self
            .phase_list
            .iter()
            .map(|p| p.get_time())
            .sum::<Duration>()
            - self
                .phase_list
                .last()
                .map(|p| p.get_time())
                .unwrap_or_default();
        self.phase_list
            .last_mut()
            .map(|p| p.0.lock().map(|mut p| p.set_time(time - diff)));
    }

    fn add_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            let _ = p.0.try_lock().map(|mut p| p.add_time(dur));
        });
    }

    fn rem_time(&mut self, dur: Duration) {
        self.phase_list.last_mut().map(|p| {
            let _ = p.0.try_lock().map(|mut p| p.rem_time(dur));
        });
    }

    fn get_progress(&self) -> f64 {
        return 1.0
            - (1.0 - 1.0_f64 / self.get_hunt_type().get_odds() as f64).powi(self.get_rolls());
    }

    fn is_active(&self) -> bool {
        for p in self.phase_list.iter() {
            if p.try_lock().map(|p| p.is_active()).unwrap_or_default() {
                return true;
            }
        }
        return false;
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
        } else {
            self.phase_list
                .last_mut()
                .map(|p| p.try_lock().ok())
                .flatten()
                .map(|mut p| p.set_active(active));
        }
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
        let _ = self.phase_list.iter_mut().for_each(|c| c.set_charm(set));
    }

    fn new_phase(&mut self, id: i32, name: String) {
        self.phase_list.push(ArcCountable::new(Box::new(Phase::new(
            id,
            name,
            self.get_hunt_type(),
            self.has_charm(),
        ))))
    }

    fn new_counter(&mut self, id: i32, name: String) -> Result<ArcCountable, String> {
        let arc_counter = ArcCountable::new(Box::new(Counter::new(id, name)?));
        self.phase_list.push(arc_counter.clone());
        return Ok(arc_counter);
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Phase {
    pub id: i32,
    pub name: String,
    pub count: i32,
    pub time: Duration,
    pub is_active: bool,
    pub hunt_type: Hunttype,
    pub has_charm: bool,
}

impl Phase {
    fn new(id: i32, name: impl ToString, hunt_type: Hunttype, has_charm: bool) -> Self {
        return Phase {
            id,
            name: name.to_string(),
            count: 0,
            time: Duration::ZERO,
            is_active: false,
            hunt_type,
            has_charm,
        };
    }
}

impl Countable for Phase {
    fn get_id(&self) -> i32 {
        return self.id;
    }

    fn get_uuid(&self) -> String {
        format!("p{}", self.id)
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name
    }

    fn get_count(&self) -> i32 {
        return self.count;
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

    fn get_time(&self) -> Duration {
        return self.time;
    }

    fn set_time(&mut self, dur: Duration) {
        self.time = dur
    }

    fn add_time(&mut self, dur: Duration) {
        self.time += dur
    }

    fn rem_time(&mut self, dur: Duration) {
        self.time -= dur
    }

    fn get_progress(&self) -> f64 {
        return 1.0 - (1.0 - 1.0_f64 / self.hunt_type.get_odds() as f64).powi(self.get_rolls());
    }

    fn is_active(&self) -> bool {
        return self.is_active;
    }

    fn toggle_active(&mut self) {
        self.is_active = !self.is_active;
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
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

    fn new_phase(&mut self, _: i32, _: String) {
        return ();
    }

    fn new_counter(&mut self, _: i32, _: String) -> Result<ArcCountable, String> {
        return Err(String::from("Can not add counter to phase"));
    }

    fn get_phases(&self) -> Vec<&ArcCountable> {
        return vec![];
    }

    fn get_phases_mut(&mut self) -> Vec<&mut ArcCountable> {
        return vec![];
    }

    fn has_children(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

cfg_if::cfg_if!(
    if #[cfg(feature = "ssr")] {
        use backend::{DbCounter, DbPhase};
        impl SerCounter {
            pub async fn from_db(username: String, token: String, value: DbCounter) -> Self {
                let mut phase_list = Vec::new();
                for id in value.phases {
                    if let Ok(phase) = get_phase_by_id(username.clone(), token.clone(), id).await {
                        phase_list.push(phase)
                    }
                }

                Self {
                    id: value.id,
                    name: value.name,
                    phase_list,
                }
            }
            pub async fn to_db(&self, user_id: i32) -> DbCounter {
                DbCounter {
                    id: self.id,
                    user_id,
                    name: self.name.clone(),
                    phases: self.phase_list.iter().map(|p| p.id).collect()
                }
            }
        }

        impl From<DbPhase> for Phase {
            fn from(value: DbPhase) -> Self {
                Self {
                    id: value.id,
                    name: value.name,
                    count: value.count,
                    time: Duration::from_millis(value.time as u64),
                    is_active: false,
                    hunt_type: value.hunt_type.into(),
                    has_charm: value.has_charm,
                }
            }
        }

        impl Phase {
            pub async fn to_db(self, user_id: i32) -> DbPhase {
                DbPhase {
                    id: self.id,
                    user_id,
                    name: self.name.clone(),
                    count: self.count,
                    time: self.time.as_millis() as i64,
                    hunt_type: self.hunt_type.into(),
                    has_charm: self.has_charm,
                }
            }
        }

        impl Into<backend::Hunttype> for Hunttype {
            fn into(self) -> backend::Hunttype {
                match self {
                    Self::OldOdds => backend::Hunttype::OldOdds,
                    Self::NewOdds => backend::Hunttype::NewOdds,
                    Self::SOS     => backend::Hunttype::SOS,
                    Self::DexNav  => backend::Hunttype::DexNav,
                }
            }
        }

        impl From<backend::Hunttype> for Hunttype {
            fn from(value: backend::Hunttype) -> Self {
                match value {
                    backend::Hunttype::OldOdds => Self::OldOdds,
                    backend::Hunttype::NewOdds => Self::NewOdds,
                    backend::Hunttype::SOS => Self::SOS,
                    backend::Hunttype::DexNav => Self::DexNav,
                }
            }
        }
    }
);
