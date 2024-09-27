#![allow(dead_code)]

use crate::models::base::diff;

use diesel::associations::HasTable;
use diesel::expression::{NonAggregate, SelectableExpression};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::{AsQuery, IntoUpdateTarget, QueryFragment, QueryId};
use diesel::{Insertable, QuerySource};
use serde::Serialize;

use diff::{Change, Diffable};

pub struct Model<M, T>
where
    T: Table,
    M: Columns,
{
    pub record: M,
    initial: Option<M>,
    table: T,
}

pub trait Columns {
    type ReturnType: QueryFragment<diesel::pg::Pg> + QueryId + NonAggregate;

    fn columns() -> Self::ReturnType;
}

impl<M, T> Model<M, T>
where
    T: Table + QueryId + 'static,
    M: Columns,

    // Needed for returning clause
    <M as Columns>::ReturnType:
        SelectableExpression<T> + QueryFragment<diesel::pg::Pg> + NonAggregate,
{
    pub fn insert(self, connection: &mut PgConnection)
    where
        M: Insertable<T>,
        <M as diesel::Insertable<T>>::Values: diesel::query_builder::QueryId
            + diesel::query_builder::QueryFragment<diesel::pg::Pg>
            + diesel::insertable::CanInsertInSingleQuery<diesel::pg::Pg>,
        <T as QuerySource>::FromClause: diesel::query_builder::QueryFragment<diesel::pg::Pg>,
    {
        use diesel::RunQueryDsl;

        diesel::insert_into(self.table)
            .values(self.record)
            .returning(M::columns())
            .execute(connection)
            .expect("Error inserting record");
    }

    pub fn update(self, connection: &mut PgConnection)
    where
        M: Diffable + AsChangeset<Target = T>,
        T: IntoUpdateTarget + HasTable<Table = T> + QuerySource + QueryFragment<Pg>,
        <T as QuerySource>::FromClause: QueryFragment<Pg>,
        <T as IntoUpdateTarget>::WhereClause: QueryFragment<Pg>,
        M::Changeset: QueryFragment<Pg>, // Add this bound for the Changeset
        diesel::query_builder::UpdateStatement<
            T,
            <T as IntoUpdateTarget>::WhereClause,
            M::Changeset,
        >: AsQuery,
    {
        use diesel::RunQueryDsl;

        diesel::update(self.table)
            .set(self.record)
            .execute(connection)
            .expect("Error updating record");
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

impl<M, T> Model<M, T>
where
    T: Table,
    M: Columns + Clone + Serialize,
{
    pub fn new(record: M, table: T) -> Self {
        Model {
            record: record.clone(),
            initial: Some(record),
            table,
        }
    }

    pub fn insertable(record: M, table: T) -> Self {
        Model {
            record,
            initial: None,
            table,
        }
    }

    pub fn is_new(&self) -> bool {
        self.initial.is_none()
    }

    pub fn record(&self) -> &M {
        &self.record
    }

    pub fn initial(&self) -> &Option<M> {
        &self.initial
    }

    fn changes(&self, is_delete: bool) -> Option<Change>
    where
        M: Diffable,
    {
        if is_delete {
            return Some(Change::Delete(serde_json::to_value(&self.record).unwrap()));
        }

        match self.initial {
            None => Some(Change::Insert(serde_json::to_value(&self.record).unwrap())),
            Some(ref initial) => self.record.diff(initial).map(Change::Update),
        }
    }

    pub fn save(&mut self)
    where
        M: Diffable,
    {
        let changes = self.changes(false);

        match changes {
            Some(Change::Insert(_)) => {
                // self.record.save();
                self.initial = Some(self.record.clone());
            }
            Some(Change::Update(_)) => {
                // self.record.update();
                self.initial = Some(self.record.clone());
            }
            _ => (),
        }
    }

    pub fn delete(&mut self)
    where
        M: Diffable,
    {
        if self.is_new() {
            // Nothing to delete
            return;
        }

        // self.record.delete();
        self.initial = None;

        let _changes = self.changes(true);
    }
}
