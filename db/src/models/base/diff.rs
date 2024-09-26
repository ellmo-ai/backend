#![allow(dead_code)]

use serde::Serialize;
use std::collections::BTreeMap;

pub type Diff = BTreeMap<String, DiffItem>;
pub struct DiffItem {
    pub(crate) before: serde_json::Value,
    pub(crate) after: serde_json::Value,
}

pub enum Change {
    Insert(serde_json::Value),
    Update(Diff),
    Delete(serde_json::Value),
}

pub trait Diffable: Serialize + Clone {
    fn diff(&self, other: &Self) -> Option<Diff>;
}
