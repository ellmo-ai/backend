#![allow(unused_imports)]
#![allow(dead_code)]

use crate::models::base::diff;

use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::query_builder::{AsQuery, QueryId};
use diesel::{Insertable, QuerySource};
use serde::Serialize;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use diff::{Change, Diff, DiffItem, Diffable};

pub struct Model<M, T>
where
    T: Table,
    M: Diffable + Insertable<T>,
{
    pub record: M,
    initial: Option<M>,
    table: T,
}

impl<M, T> Model<M, T>
where
    T: Table + QueryId + 'static,
    M: Diffable + Insertable<T>,
    // Needed for insert
    <M as diesel::Insertable<T>>::Values: diesel::query_builder::QueryId,
    <M as diesel::Insertable<T>>::Values: diesel::query_builder::QueryFragment<diesel::pg::Pg>,
    <T as QuerySource>::FromClause: diesel::query_builder::QueryFragment<diesel::pg::Pg>,
    <M as diesel::Insertable<T>>::Values:
        diesel::insertable::CanInsertInSingleQuery<diesel::pg::Pg>,
{
    fn insert(self, connection: &mut PgConnection) {
        use diesel::RunQueryDsl;

        diesel::insert_into(self.table)
            .values(self.record)
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
    M: Diffable + Insertable<T>,
{
    pub fn new(record: M, table: T) -> Self {
        Model {
            record: record.clone(),
            initial: Some(record),
            table,
            // _marker: PhantomData,
        }
    }

    pub fn insertable(record: M, table: T) -> Self {
        Model {
            record,
            initial: None,
            table,
            // _marker: PhantomData,
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[derive(Serialize, Clone)]
//     struct TestModel {
//         id: i32,
//         name: String,
//     }
//
//     impl Diffable for TestModel {
//         fn diff(&self, other: &Self) -> Option<Diff> {
//             let mut diff = BTreeMap::new();
//
//             if self.name != other.name {
//                 diff.insert(
//                     "name".to_string(),
//                     DiffItem {
//                         before: serde_json::Value::String(self.name.clone()),
//                         after: serde_json::Value::String(other.name.clone()),
//                     },
//                 );
//             }
//
//             if diff.is_empty() {
//                 None
//             } else {
//                 Some(diff)
//             }
//         }
//     }
//
//     #[test]
//     fn basic() {
//         let test_table = table! {
//             test_model (id) {
//                 id -> Int4,
//                 name -> Text,
//             }
//         };
//
//         impl ModelLifecycle<test_table> for TestModel {
//             fn save(&mut self) {
//                 // Save the record
//                 // Update the initial record
//             }
//
//             fn update(&mut self) {
//                 // Update the record
//                 // Update the initial record
//             }
//
//             fn delete(&mut self) {
//                 // Delete the record
//                 // Update the initial record
//             }
//         }
//
//         let e = TestModel {
//             id: 1,
//             name: "foo".to_string(),
//         };
//
//         let mut model = Model::new(e, test_table);
//     }
// }
