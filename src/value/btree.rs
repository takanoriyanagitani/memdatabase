use std::collections::{BTreeMap, BTreeSet, VecDeque};

use prost_types::Value;

#[derive(Clone)]
pub enum Val {
    Var(Value),
    Map(BTreeMap<Vec<u8>, Value>),
    Set(BTreeSet<Vec<u8>>),
    Deq(VecDeque<Value>),
}
