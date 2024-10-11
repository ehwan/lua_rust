use std::cell::RefCell;
use std::rc::Rc;

use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

/// init table module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut table = LuaTable::new();
    Ok(LuaValue::Table(Rc::new(RefCell::new(table))))
}

// @TODO
