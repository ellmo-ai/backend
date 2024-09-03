use diesel::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::models::repository::{DieselRepository, Repository};
use crate::schema::test_registration::dsl::test_registration;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::test_registration)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct TestRegistration {
    pub id: i32,
    pub blob_url: String,
    pub hash: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

type TestId = String;

pub type Metadata = HashMap<TestId, Vec<Test>>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Test {
    pub version: String,
    pub export_name: String,
    pub file_path: String,
}

#[derive(Insertable, Selectable, Queryable)]
#[diesel(table_name = crate::schema::test_registration)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableTestRegistration {
    pub blob_url: String,
    pub metadata: serde_json::Value,
    pub hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<'a> Repository for DieselRepository<'a, test_registration> {
    type Entity = TestRegistration;
    type InsertableEntity = InsertableTestRegistration;
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
            .returning(crate::schema::test_registration::all_columns)
            .get_result(self.connection)
    }

    fn delete(&mut self, id: Self::Id) -> QueryResult<()> {
        diesel::delete(self.table.find(id))
            .execute(self.connection)
            .map(|_| ())
    }
}
