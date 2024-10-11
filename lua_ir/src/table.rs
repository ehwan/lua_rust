use crate::LuaValue;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LuaTable {
    pub(crate) map: HashMap<LuaValue, LuaValue>,
}
impl LuaTable {
    pub fn with_capacity(capacity: usize) -> Self {
        LuaTable {
            map: HashMap::with_capacity(capacity),
        }
    }
    pub fn new() -> Self {
        LuaTable {
            map: HashMap::new(),
        }
    }
}
