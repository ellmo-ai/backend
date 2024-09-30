#![allow(dead_code)]

use crate::models::base::diff;

use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::query_builder::{AsQuery, InsertStatement, IntoUpdateTarget, QueryFragment, QueryId};
use diesel::query_dsl::LoadQuery;
use diesel::{Insertable, QuerySource};
use serde::Serialize;

use crate::models::base::diff::Change;
use diff::Diffable;

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("Diesel error: {0}")]
    QueryError(#[from] diesel::result::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// A helper struct for working with Diesel models
/// # Type parameters
/// - `Mod`: The model type
/// - `Tab`: The table type
pub struct Model<Mod, Tab>
where
    Tab: diesel::Table,
{
    /// The current record, whether new or loaded from the database
    pub record: Mod,
    /// The initial state of the model, if it was loaded from the database
    initial: Option<Mod>,
    /// The table associated with the model
    table: Tab,
}

impl<Ins, Tab> Model<Ins, Tab>
where
    Tab: diesel::Table + QueryId + 'static + Clone,
    Ins: Insertable<Tab> + Clone + ModelLifecycle<Tab>,
{
    pub async fn insert<Mod>(
        &mut self,
        connection: &mut PgConnection,
    ) -> Result<Model<Mod, Tab>, ModelError>
    where
        Ins: Serialize,
        Mod: Queryable<Tab::SqlType, diesel::pg::Pg> + Clone + ModelLifecycle<Tab>,
        <Ins as Insertable<Tab>>::Values: diesel::query_builder::QueryId
            + diesel::query_builder::QueryFragment<diesel::pg::Pg>
            + diesel::insertable::CanInsertInSingleQuery<diesel::pg::Pg>,
        <Tab as QuerySource>::FromClause: diesel::query_builder::QueryFragment<diesel::pg::Pg>,
        Tab: IntoUpdateTarget,
        Tab: HasTable<Table = Tab> + QuerySource + QueryFragment<diesel::pg::Pg>,
        InsertStatement<Tab, <Ins as Insertable<Tab>>::Values>:
            LoadQuery<'static, PgConnection, Mod>,
    {
        use diesel::RunQueryDsl;

        self.record.before_insert();

        let mut res = diesel::insert_into(Tab::table())
            .values(self.record.clone())
            .get_result::<Mod>(connection)
            .map_err(ModelError::QueryError)?;

        res.after_insert();

        let change = Change::Insert(
            serde_json::to_value(&self.record).map_err(ModelError::SerializationError)?,
        );
        change.sync().await;

        // Update initial and record with the result
        Ok(Model {
            record: res.clone(),
            initial: Some(res),
            table: self.table.clone(),
        })
    }
}

impl<Mod, Tab> Model<Mod, Tab>
where
    Tab: diesel::Table + QueryId + 'static + IntoUpdateTarget,
    Mod: Diffable
        + AsChangeset<Target = Tab>
        + Queryable<Tab::SqlType, diesel::pg::Pg>
        + Clone
        + ModelLifecycle<Tab>,
{
    pub async fn update(&mut self, connection: &mut PgConnection) -> Result<(), ModelError>
    where
        Tab: HasTable<Table = Tab> + QuerySource + QueryFragment<diesel::pg::Pg>,
        <Tab as QuerySource>::FromClause: QueryFragment<diesel::pg::Pg>,
        <Tab as IntoUpdateTarget>::WhereClause: QueryFragment<diesel::pg::Pg>,
        Mod::Changeset: QueryFragment<diesel::pg::Pg>,
        diesel::query_builder::UpdateStatement<
            Tab,
            <Tab as IntoUpdateTarget>::WhereClause,
            Mod::Changeset,
        >: LoadQuery<'static, PgConnection, Mod> + AsQuery,
    {
        use diesel::RunQueryDsl;

        self.record.before_update();

        let mut res = diesel::update(Tab::table())
            .set(self.record.clone())
            .get_result::<Mod>(connection)
            .map_err(ModelError::QueryError)?;

        res.after_update();

        // Update initial and record with the result
        self.initial = Some(res.clone());
        self.record = res;

        self.changes(false)
            .map(|changes| async move { changes.sync().await });

        Ok(())
    }
}

pub trait ModelLifecycle<T: diesel::Table> {
    /// Called before inserting the model
    fn before_insert(&mut self) {}
    /// Called before updating the model
    fn before_update(&mut self) {}
    /// Called before deleting the model
    fn before_delete(&mut self) {}
    /// Called after inserting the model
    fn after_insert(&mut self) {}
    /// Called after updating the model
    fn after_update(&mut self) {}
    /// Called after deleting the model
    fn after_delete(&mut self) {}
}

impl<Mod, Tab> Model<Mod, Tab>
where
    Mod: Clone,
    Tab: Table,
{
    pub fn new(record: Mod, table: Tab) -> Self {
        Model {
            record: record.clone(),
            initial: Some(record),
            table,
        }
    }

    pub fn insertable(record: Mod, table: Tab) -> Self
    where
        Mod: Insertable<Tab>,
    {
        Model {
            record,
            initial: None,
            table,
        }
    }

    pub fn is_new(&self) -> bool {
        self.initial.is_none()
    }

    pub fn record(&self) -> &Mod {
        &self.record
    }

    pub fn initial(&self) -> &Option<Mod> {
        &self.initial
    }

    fn changes(&self, is_delete: bool) -> Option<Change>
    where
        Mod: Diffable,
    {
        if is_delete {
            return Some(Change::Delete(serde_json::to_value(&self.record).unwrap()));
        }

        match self.initial {
            None => Some(Change::Insert(serde_json::to_value(&self.record).unwrap())),
            Some(ref initial) => self.record.diff(initial).map(Change::Update),
        }
    }
}
