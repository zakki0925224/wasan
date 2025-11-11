use crate::execution::{store::Store, value::Value};
use std::collections::HashMap;

pub type ImportFunc = Box<dyn FnMut(&mut Store, Vec<Value>) -> anyhow::Result<Option<Value>>>;
pub type Import = HashMap<String, HashMap<String, ImportFunc>>;
