use super::*;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CountableDiff {
    pub id: CountableId,

    pub count: i32,
    pub time: chrono::TimeDelta,

    has_change: bool,
}

impl CountableDiff {
    pub fn new(countable: CountableId) -> Self {
        Self {
            id: countable,
            count: 0,
            time: chrono::TimeDelta::zero(),
            has_change: false,
        }
    }

    pub fn increase(&mut self, step_size: i32) -> Self {
        self.count += step_size;

        return *self;
    }
}

impl Savable for CountableDiff {
    fn has_change(&self) -> bool {
        return self.has_change;
    }
}

impl LocalSavable for CountableDiff {
    const INDEXED_DB_NAME: &'static str = "CountableDiff";

    fn idb_key(&self) -> String {
        self.id.0.to_string()
    }
}
