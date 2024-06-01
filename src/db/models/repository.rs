use diesel::QueryResult;

pub trait Repository {
    type Entity;
    type Id;

    fn find_all(&mut self) -> QueryResult<Vec<Self::Entity>>;
    fn find_by_id(&mut self, id: Self::Id) -> QueryResult<Self::Entity>;
    fn create(&mut self, entity: &Self::Entity) -> QueryResult<()>;
    fn delete(&mut self, id: Self::Id) -> QueryResult<()>;
}

pub struct DieselRepository<'a, T> {
    pub connection: &'a mut diesel::PgConnection,
    pub table: T,
}
