use diesel::QueryResult;

#[allow(dead_code)]
pub trait Repository {
    type Entity;
    type InsertableEntity;
    type Id;

    fn find_all(&mut self) -> QueryResult<Vec<Self::Entity>>;
    fn find_by_id(&mut self, id: Self::Id) -> QueryResult<Self::Entity>;
    fn create(&mut self, entity: &Self::InsertableEntity) -> QueryResult<Self::Entity>;
    fn delete(&mut self, id: Self::Id) -> QueryResult<()>;
}

pub struct DieselRepository<'a, T>
where
    T: diesel::Table,
{
    pub connection: &'a mut diesel::PgConnection,
    pub table: T,
}

impl<'a, T> DieselRepository<'a, T>
where
    T: diesel::Table,
{
    pub fn new(connection: &'a mut diesel::PgConnection, table: T) -> Self {
        DieselRepository { connection, table }
    }
}
