use crate::IntType;
use crate::{LuaNumber, LuaValue};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LuaTable {
    /// every key except number goes here
    map: HashMap<LuaValue, LuaValue>,
    /// every key with number goes here
    arr: BTreeMap<i64, LuaValue>,

    /// metatable
    pub(crate) meta: Option<Rc<RefCell<LuaTable>>>,
}
impl LuaTable {
    pub fn with_capacity(capacity: usize) -> Self {
        LuaTable {
            map: HashMap::with_capacity(capacity),
            arr: BTreeMap::new(),
            meta: None,
        }
    }
    pub fn new() -> Self {
        LuaTable {
            map: HashMap::new(),
            arr: BTreeMap::new(),
            meta: None,
        }
    }
    pub fn get_metavalue(&self, key: &str) -> Option<LuaValue> {
        if let Some(meta) = &self.meta {
            meta.borrow().map.get(&key.into()).cloned()
        } else {
            None
        }
    }
    /// get value from table.
    /// key can be any lua value.
    pub fn get(&self, key: &LuaValue) -> Option<&LuaValue> {
        match key {
            LuaValue::Number(LuaNumber::Int(n)) => self.arr.get(n),
            _ => self.map.get(key),
        }
    }

    /// get value from table.
    /// key can be any lua value.
    pub fn get_mut(&mut self, key: &LuaValue) -> Option<&mut LuaValue> {
        match key {
            LuaValue::Number(LuaNumber::Int(n)) => self.arr.get_mut(n),
            _ => self.map.get_mut(key),
        }
    }

    /// get value from array part of table.
    pub fn get_arr(&self, key: IntType) -> Option<&LuaValue> {
        self.arr.get(&key)
    }
    /// get value from array part of table.
    pub fn get_arr_mut(&mut self, key: IntType) -> Option<&mut LuaValue> {
        self.arr.get_mut(&key)
    }

    /// get value from hash part of table.
    pub fn get_table(&self, key: &LuaValue) -> Option<&LuaValue> {
        self.map.get(key)
    }
    /// get value from hash part of table.
    pub fn get_table_mut(&mut self, key: &LuaValue) -> Option<&mut LuaValue> {
        self.map.get_mut(key)
    }

    pub fn insert(&mut self, key: LuaValue, value: LuaValue) {
        match key {
            LuaValue::Nil => {}
            LuaValue::Number(LuaNumber::Int(n)) => {
                self.arr.insert(n, value);
            }
            _ => {
                self.map.insert(key, value);
            }
        }
    }

    /// get length of array part of table.
    pub fn len(&self) -> IntType {
        self.arr.last_key_value().map_or(0, |(k, _)| *k)
    }
}
