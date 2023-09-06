use chrono;

pub struct DbCounter {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub phases: Vec<i32>,
}

pub struct DbPhase {
    pub id: i32,
    pub name: String,
    pub count: i32,
    pub time: i64,
}
