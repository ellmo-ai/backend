use diesel::prelude::*;
use crate::db::models::repository::{DieselRepository, Repository};
use crate::db::schema::spans::dsl::spans;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::db::schema::spans)]
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

impl<'a> Repository for DieselRepository<'a, spans> {
    type Entity = Span;
    type Id = i32;

    fn find_all(&mut self) -> QueryResult<Vec<Self::Entity>> {
        self.table.load::<Span>(self.connection)
    }

    fn find_by_id(&mut self, id: Self::Id) -> QueryResult<Self::Entity> {
        self.table.find(id).get_result::<Span>(self.connection)
    }

    fn create(&mut self, entity: &Self::Entity) -> QueryResult<()> {
        diesel::insert_into(self.table).values(entity).execute(self.connection).map(|_| ())
    }

    fn delete(&mut self, id: Self::Id) -> QueryResult<()> {
        diesel::delete(self.table.find(id)).execute(self.connection).map(|_| ())
    }
}