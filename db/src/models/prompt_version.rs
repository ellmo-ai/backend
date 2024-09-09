use crate::models::repository::{DieselRepository, Repository};
use crate::schema::prompt_version::dsl::prompt_version;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::prompt_version)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct PromptVersion {
    pub id: i32,
    pub name: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable, Selectable, Queryable)]
#[diesel(table_name = crate::schema::prompt_version)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertablePromptVersion {
    pub name: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<'a> Repository for DieselRepository<'a, prompt_version> {
    type Entity = PromptVersion;
    type InsertableEntity = InsertablePromptVersion;
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
            .returning(crate::schema::prompt_version::all_columns)
            .get_result(self.connection)
    }

    fn delete(&mut self, id: Self::Id) -> QueryResult<()> {
        diesel::delete(self.table.find(id))
            .execute(self.connection)
            .map(|_| ())
    }
}
