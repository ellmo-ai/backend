#![allow(unused_imports)]
#![allow(dead_code)]

use crate::models::base::diff;

use diesel::associations::HasTable;
use diesel::expression::ValidGrouping;
use diesel::prelude::*;
use diesel::query_builder::{AsQuery, QueryFragment, QueryId};
use diesel::{Insertable, QuerySource};
use serde::Serialize;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use diff::{Change, Diff, DiffItem, Diffable};

pub struct Model<M, T>
where
    T: Table,
    M: Diffable + Insertable<T> + Columns,
{
    pub record: M,
    initial: Option<M>,
    table: T,
}

use diesel::expression::is_aggregate::No;
use diesel::expression::{NonAggregate, SelectableExpression};

pub trait Columns {
    type ReturnType: QueryFragment<diesel::pg::Pg>
        + QueryId
        + SelectableExpression<Self::Table>
        + NonAggregate;
    type Table: Table;

    fn columns() -> Self::ReturnType;
}

impl<M, T> Model<M, T>
where
    T: Table + QueryId + 'static,
    M: Diffable + Insertable<T> + Columns<Table = T>,

    // Needed for returning clause
    <M as Columns>::ReturnType:
        SelectableExpression<T> + QueryFragment<diesel::pg::Pg> + NonAggregate,
{
    fn insert(self, connection: &mut PgConnection)
    where
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
    M: Diffable + Insertable<T> + Columns,
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

    fn changes(&self, is_delete: bool) -> Option<Change> {
        if is_delete {
            return Some(Change::Delete(serde_json::to_value(&self.record).unwrap()));
        }

        match self.initial {
            None => Some(Change::Insert(serde_json::to_value(&self.record).unwrap())),
            Some(ref initial) => self.record.diff(initial).map(Change::Update),
        }
    }

    pub fn save(&mut self) {
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

    pub fn delete(&mut self) {
        if self.is_new() {
            // Nothing to delete
            return;
        }

        // self.record.delete();
        self.initial = None;

        let _changes = self.changes(true);
    }
}
