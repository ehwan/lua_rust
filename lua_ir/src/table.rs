use crate::{LuaValue, RuntimeError};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LuaTable {
    pub(crate) map: HashMap<LuaValue, LuaValue>,
    pub(crate) meta: Option<Rc<RefCell<LuaTable>>>,
}
impl LuaTable {
    pub fn with_capacity(capacity: usize) -> Self {
        LuaTable {
            map: HashMap::with_capacity(capacity),
            meta: None,
        }
    }
    pub fn new() -> Self {
        LuaTable {
            map: HashMap::new(),
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

    pub fn len(&self) -> Result<usize, RuntimeError> {
        // @TODO
        unimplemented!("table length");
    }
}
