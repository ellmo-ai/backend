use crate::db::models::repository::{DieselRepository, Repository};
use crate::db::schema::span::dsl::span;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::span)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Span {
    pub id: i32,
    pub ts_start: chrono::DateTime<chrono::Utc>,
    pub ts_end: chrono::DateTime<chrono::Utc>,
    pub operation_name: String,
    pub parent_span_id: Option<i32>,
    pub external_uuid: Option<uuid::Uuid>,
}

#[derive(Insertable, Selectable, Queryable)]
#[diesel(table_name = crate::db::schema::span)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableSpan {
    pub ts_start: chrono::DateTime<chrono::Utc>,
    pub ts_end: chrono::DateTime<chrono::Utc>,
    pub operation_name: String,
    pub parent_span_id: Option<i32>,
    pub external_uuid: Option<uuid::Uuid>,
}

impl<'a> Repository for DieselRepository<'a, span> {
    type Entity = Span;
    type InsertableEntity = InsertableSpan;
    type Id = i32;

    fn find_all(&mut self) -> QueryResult<Vec<Self::Entity>> {
        self.table.load::<Span>(self.connection)
    }

    fn find_by_id(&mut self, id: Self::Id) -> QueryResult<Self::Entity> {
        self.table.find(id).get_result::<Span>(self.connection)
    }

    fn create(&mut self, entity: &Self::InsertableEntity) -> QueryResult<Self::Entity> {
        diesel::insert_into(self.table)
            .values(entity)
            .returning(crate::db::schema::span::all_columns)
            .get_result(self.connection)
    }

    fn delete(&mut self, id: Self::Id) -> QueryResult<()> {
        diesel::delete(self.table.find(id))
            .execute(self.connection)
            .map(|_| ())
    }
}
