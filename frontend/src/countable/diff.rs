use super::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CountableDiff {
    Counter {
        uuid: uuid::Uuid,
        owner_uuid: uuid::Uuid,
        parent: Option<CountableId>,
        name: String,
        last_edit: chrono::NaiveDateTime,
        is_deleted: bool,
    },

    Phase {
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
    },
}

impl ToJsValue for CountableDiff {
    const OBJECT_STORE: &'static str = "Countable";

    fn to_js_value(&self) -> AppResult<wasm_bindgen::JsValue> {
        todo!()
    }
}
