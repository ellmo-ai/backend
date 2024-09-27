use crate::models::base::diff::{Diff, Diffable};
use crate::models::base::model::{Columns, Model};

use crate::models::repository::{DieselRepository, Repository};
use crate::schema::eval::dsl::eval;
use diesel::prelude::*;
use serde::Serialize;

// GOALS
// Need a way to track changes to the model
// Need a way to add middleware logic for updating/saving/deleting

#[derive(Serialize, Clone, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::eval)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Eval {
    pub id: i32,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub prompt_version_id: i32,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = crate::schema::eval)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableEval {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub prompt_version_id: i32,
}

impl Columns for InsertableEval {
    type ReturnType = (
        crate::schema::eval::id,
        crate::schema::eval::name,
        crate::schema::eval::created_at,
        crate::schema::eval::prompt_version_id,
    );

    fn columns() -> Self::ReturnType {
        (
            crate::schema::eval::id,
            crate::schema::eval::name,
            crate::schema::eval::created_at,
            crate::schema::eval::prompt_version_id,
        )
    }
}

impl Columns for Eval {
    type ReturnType = (
        crate::schema::eval::id,
        crate::schema::eval::name,
        crate::schema::eval::created_at,
        crate::schema::eval::prompt_version_id,
    );

    fn columns() -> Self::ReturnType {
        (
            crate::schema::eval::id,
            crate::schema::eval::name,
            crate::schema::eval::created_at,
            crate::schema::eval::prompt_version_id,
        )
    }
}

impl Diffable for Eval {
    fn diff(&self, _other: &Self) -> Option<Diff> {
        None
    }
}

#[allow(dead_code)]
fn foo() {
    let e = Eval {
        id: 1,
        name: "foo".to_string(),
        created_at: chrono::Utc::now(),
        prompt_version_id: 1,
    };

    let model: Model<Eval, InsertableEval, crate::schema::eval::table> =
        Model::new(e, crate::schema::eval::table);
    let mut connection = crate::establish_connection();
    let _ = model.insert(&mut connection);

    // let e2 = Eval {
    //     id: 1,
    //     name: "foo".to_string(),
    //     created_at: chrono::Utc::now(),
    //     prompt_version_id: 1,
    // };
    //
    // let model = Model::new(e2, crate::schema::eval::table);
    // model.update(&mut connection);
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

    fn update(&mut self, id: Self::Id, entity: &Self::Entity) -> QueryResult<Self::Entity> {
        diesel::update(self.table.find(id))
            .set(entity)
            .returning(crate::schema::eval::all_columns)
            .get_result(self.connection)
    }

    fn delete(&mut self, id: Self::Id) -> QueryResult<()> {
        diesel::delete(self.table.find(id))
            .execute(self.connection)
            .map(|_| ())
    }
}
