use crate::models::repository::{DieselRepository, Repository};
use crate::schema::eval_result::dsl::eval_result;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::eval_result)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EvalResult {
    pub id: i32,
    pub eval_version_id: i32,
    pub scores: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable, Selectable, Queryable)]
#[diesel(table_name = crate::schema::eval_result)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableEvalResult {
    pub eval_version_id: i32,
    pub scores: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SingleEvalScore {
    pub eval_hash: String,
    pub score: f32,
}

pub type EvalRunScores = Vec<SingleEvalScore>;

impl<'a> Repository for DieselRepository<'a, eval_result> {
    type Entity = EvalResult;
    type InsertableEntity = InsertableEvalResult;
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
            .returning(crate::schema::eval_result::all_columns)
            .get_result(self.connection)
    }

    fn delete(&mut self, id: Self::Id) -> QueryResult<()> {
        diesel::delete(self.table.find(id))
            .execute(self.connection)
            .map(|_| ())
    }
}
