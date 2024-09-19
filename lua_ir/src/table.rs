use crate::LuaValue;
use crate::RuntimeError;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LuaTable {
    pub map: HashMap<String, LuaValue>,
    pub meta: HashMap<String, LuaValue>,
}
impl LuaTable {
    pub fn with_capacity(capacity: usize) -> Self {
        LuaTable {
            map: HashMap::with_capacity(capacity),
            meta: HashMap::new(),
        }
    }
    pub fn new() -> Self {
        LuaTable {
            map: HashMap::new(),
            meta: HashMap::new(),
        }
    }

    pub fn table_index_get(&self, idx: LuaValue) -> Result<LuaValue, RuntimeError> {
        Ok(self
            .map
            .get(&idx.as_key()?)
            .unwrap_or(&LuaValue::Nil)
            .clone())
    }
    pub fn table_index_set(&mut self, idx: LuaValue, value: LuaValue) -> Result<(), RuntimeError> {
        if idx.is_nil() {
            return Ok(());
        }
        self.map.insert(idx.as_key()?, value);
        Ok(())
    }
    pub fn table_index_init(&mut self, idx: LuaValue, value: LuaValue) -> Result<(), RuntimeError> {
        if idx.is_nil() {
            return Ok(());
        }
        self.map.insert(idx.as_key()?, value);
        Ok(())
    }
}
