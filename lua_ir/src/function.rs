use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::LuaEnv;
use crate::LuaThread;
use crate::LuaValue;
use crate::RuntimeError;

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
    RustFunc(
        Box<
            dyn Fn(
                &mut LuaEnv,
                &Rc<RefCell<LuaThread>>,
                &Chunk,
                usize,
                Option<usize>,
            ) -> Result<(), RuntimeError>,
        >,
    ),
}
impl std::fmt::Debug for LuaFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaFunction::LuaFunc(func) => write!(f, "LuaFunctionLua({:?})", func),
            LuaFunction::RustFunc(func) => {
                write!(f, "LuaFunctionRust({:p})", func.as_ref() as *const _)
            }
        }
    }
}

impl LuaFunction {
    pub fn from_func(
        func: impl Fn(
                &mut LuaEnv,
                &Rc<RefCell<LuaThread>>,
                &Chunk,
                usize,
                Option<usize>,
            ) -> Result<(), RuntimeError>
            + 'static,
    ) -> Self {
        LuaFunction::RustFunc(Box::new(func))
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
