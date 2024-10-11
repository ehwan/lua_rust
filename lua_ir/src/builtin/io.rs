use std::cell::RefCell;
use std::rc::Rc;

use crate::LuaFunction;
use crate::LuaNumber;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

/// init math module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut io = LuaTable::new();
    Ok(LuaValue::Table(Rc::new(RefCell::new(io))))
}
