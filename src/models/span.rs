use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::spans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Span {
    pub id: i32,
    pub ts_start: chrono::DateTime<chrono::Utc>,
    pub ts_end: chrono::DateTime<chrono::Utc>,
    pub operation_name: String,
    pub attribute_ids: Vec<Option<i32>>,
    pub event_ids: Vec<Option<i32>>,
    pub link_ids: Vec<Option<i32>>,
}
