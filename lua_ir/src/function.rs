use std::boxed::Box;
use std::sync::Arc;
use std::sync::RwLock;

use crate::Chunk;
use crate::LuaEnv;
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
pub enum LuaFunction {
    /// functions written in Lua
    LuaFunc(LuaFunctionLua),
    /// built-in functions written in Rust
    RustFunc(Box<dyn Fn(&mut Stack, &mut LuaEnv, &Chunk, usize) -> Result<usize, RuntimeError>>),
}

impl std::fmt::Debug for LuaFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaFunction::LuaFunc(func) => func.fmt(f),
            LuaFunction::RustFunc(func) => write!(f, "RustFunc: {:p}", func.as_ref() as *const _),
        }
    }
}

impl LuaFunction {
    pub fn from_func(
        func: impl Fn(&mut Stack, &mut LuaEnv, &Chunk, usize) -> Result<usize, RuntimeError> + 'static,
    ) -> Self {
        LuaFunction::RustFunc(Box::new(func))
    }
}

/// functions written in Lua
#[derive(Debug, Clone)]
pub struct LuaFunctionLua {
    /// upvalues for this function object
    pub upvalues: Vec<Arc<RwLock<LuaValue>>>,
    /// actual set of instructions connected to this function object
    pub function_id: usize,
}
