use crate::models::repository::{DieselRepository, Repository};
use crate::schema::eval::dsl::eval;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::eval)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Eval {
    pub id: i32,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub prompt_version_id: i32,
}

#[derive(Insertable, Selectable, Queryable)]
#[diesel(table_name = crate::schema::eval)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableEval {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub prompt_version_id: i32,
}

impl<'a> Repository for DieselRepository<'a, eval> {
    type Entity = Eval;
    type InsertableEntity = InsertableEval;
    type Id = i32;

    fn find_all(&mut self) -> QueryResult<Vec<Self::Entity>> {
        self.table.load::<Self::Entity>(self.connection)
    }

    fn find_by_id(&mut self, id: Self::Id) -> QueryResult<Self::Entity> {
        self.table
            .find(id)
            .get_result::<Self::Entity>(self.connection)
    }

    fn create(&mut self, entity: &Self::InsertableEntity) -> QueryResult<Self::Entity> {
        diesel::insert_into(self.table)
            .values(entity)
            .returning(crate::schema::eval::all_columns)
            .get_result(self.connection)
    }

    fn delete(&mut self, id: Self::Id) -> QueryResult<()> {
        diesel::delete(self.table.find(id))
            .execute(self.connection)
            .map(|_| ())
    }
}
