#![allow(dead_code)]

use crate::models::base::diff;

use diesel::associations::HasTable;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::{InsertStatement, IntoUpdateTarget, QueryId};
use diesel::query_dsl::LoadQuery;
use diesel::{Insertable, QuerySource};

use diff::Diffable;

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("Record already exists")]
    AlreadyExists,
    #[error("Diesel error: {0}")]
    QueryError(#[from] diesel::result::Error),
}

enum ModelType<Mod, Ins> {
    Insertable(Ins),
    Model(Mod),
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
    Tab: diesel::Table + QueryId + 'static,
    Ins: Insertable<Tab>,
{
    pub fn insert<Mod>(self, connection: &mut PgConnection) -> Result<Mod, ModelError>
    where
        Mod: Queryable<Tab::SqlType, diesel::pg::Pg>,
        <Ins as Insertable<Tab>>::Values: diesel::query_builder::QueryId
            + diesel::query_builder::QueryFragment<diesel::pg::Pg>
            + diesel::insertable::CanInsertInSingleQuery<diesel::pg::Pg>,
        <Tab as QuerySource>::FromClause: diesel::query_builder::QueryFragment<diesel::pg::Pg>,
        Tab: IntoUpdateTarget,
        InsertStatement<Tab, <Ins as Insertable<Tab>>::Values>:
            LoadQuery<'static, PgConnection, Mod>,
    {
        use diesel::RunQueryDsl;

        let res = diesel::insert_into(self.table)
            .values(self.record)
            .get_result::<Mod>(connection)
            .map_err(ModelError::QueryError)?;

        Ok(res)
    }
}

pub trait ModelLifecycle<T: diesel::Table> {
    fn before_save(&mut self) {}
    fn before_update(&mut self) {}
    fn before_delete(&mut self) {}

    fn after_save(&mut self) {}
    fn after_update(&mut self) {}
    fn after_delete(&mut self) {}
}

impl<Mod, Tab> Model<Mod, Tab>
where
    Tab: Table,
{
    // pub fn new(record: Mod, table: Tab) -> Self {
    //     Model {
    //         record: ModelType::Model(record.clone()),
    //         initial: Some(record),
    //         table,
    //     }
    // }

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
    //
    // pub fn is_new(&self) -> bool {
    //     self.initial.is_none()
    // }
    //
    // pub fn record(&self) -> &Mod {
    //     &self.record
    // }
    //
    // pub fn initial(&self) -> &Option<Mod> {
    //     &self.initial
    // }
    //
    // fn changes(&self, is_delete: bool) -> Option<Change>
    // where
    //     Mod: Diffable,
    // {
    //     if is_delete {
    //         return Some(Change::Delete(serde_json::to_value(&self.record).unwrap()));
    //     }
    //
    //     match self.initial {
    //         None => Some(Change::Insert(serde_json::to_value(&self.record).unwrap())),
    //         Some(ref initial) => self.record.diff(initial).map(Change::Update),
    //     }
    // }
    //
    // pub fn save(&mut self)
    // where
    //     Mod: Diffable,
    // {
    //     let changes = self.changes(false);
    //
    //     match changes {
    //         Some(Change::Insert(_)) => {
    //             // self.record.save();
    //             self.initial = Some(self.record.clone());
    //         }
    //         Some(Change::Update(_)) => {
    //             // self.record.update();
    //             self.initial = Some(self.record.clone());
    //         }
    //         _ => (),
    //     }
    // }
    //
    // pub fn delete(&mut self)
    // where
    //     Mod: Diffable,
    // {
    //     if self.is_new() {
    //         // Nothing to delete
    //         return;
    //     }
    //
    //     // self.record.delete();
    //     self.initial = None;
    //
    //     let _changes = self.changes(true);
    // }
}
