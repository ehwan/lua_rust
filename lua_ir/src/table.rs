use crate::LuaValue;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LuaTable {
    pub map: HashMap<LuaValue, LuaValue>,
    pub meta: HashMap<LuaValue, LuaValue>,
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
}
