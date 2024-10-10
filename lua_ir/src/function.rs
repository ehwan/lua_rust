use std::cell::RefCell;
use std::rc::Rc;

use crate::LuaValue;
use crate::RuntimeError;
use crate::Stack;

/// function information
#[derive(Debug, Clone, Copy)]
pub struct FunctionInfo {
    /// number of arguments ( excluding variadic arguments )
    pub args: usize,
    /// if true, this function is variadic
    pub is_variadic: bool,
    /// necessary stack size for this function
    pub stack_size: usize,
    /// entry point of this function ( instruction index )
    pub address: usize,
}

/// lua function object
#[derive(Clone)]
pub enum LuaFunction {
    /// functions written in Lua
    LuaFunc(LuaFunctionLua),
    /// built-in functions written in Rust
    RustFunc(Rc<dyn Fn(&mut Stack, Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError>>),
}
impl std::fmt::Display for LuaFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LuaFunction")
    }
}
impl std::fmt::Debug for LuaFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LuaFunction")
    }
}

impl LuaFunction {
    pub fn from_func(
        func: impl Fn(&mut Stack, Vec<LuaValue>) -> Result<Vec<LuaValue>, RuntimeError> + 'static,
    ) -> Self {
        LuaFunction::RustFunc(Rc::new(func))
    }
}
/// functions written in Lua
#[derive(Debug, Clone)]
pub struct LuaFunctionLua {
    /// upvalues for this function object
    pub upvalues: Vec<Rc<RefCell<LuaValue>>>,
    /// actual set of instructions connected to this function object
    pub function_id: usize,
}
