use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
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
    RustFunc(Rc<dyn Fn(&mut Stack, &Chunk, usize) -> Result<usize, RuntimeError>>),
}
impl std::cmp::PartialEq for LuaFunction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LuaFunction::LuaFunc(a), LuaFunction::LuaFunc(b)) => a == b,
            (LuaFunction::RustFunc(a), LuaFunction::RustFunc(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl std::cmp::Eq for LuaFunction {}
impl std::hash::Hash for LuaFunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            LuaFunction::LuaFunc(f) => f.hash(state),
            LuaFunction::RustFunc(f) => Rc::as_ptr(f).hash(state),
        }
    }
}
impl std::fmt::Display for LuaFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaFunction::LuaFunc(func) => write!(f, "function: {:x}", func.function_id),
            LuaFunction::RustFunc(func) => write!(f, "function: {:p}", Rc::as_ptr(func)),
        }
    }
}
impl std::fmt::Debug for LuaFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaFunction::LuaFunc(func) => write!(f, "LuaFunctionLua({:?})", func),
            LuaFunction::RustFunc(func) => write!(f, "LuaFunctionRust({:p})", Rc::as_ptr(func)),
        }
    }
}

impl LuaFunction {
    pub fn from_func(
        func: impl Fn(&mut Stack, &Chunk, usize) -> Result<usize, RuntimeError> + 'static,
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
impl std::cmp::PartialEq for LuaFunctionLua {
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!("lua function comparison");
        // self.function_id == other.function_id
    }
}
impl std::cmp::Eq for LuaFunctionLua {}
impl std::hash::Hash for LuaFunctionLua {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        unimplemented!("lua function hash");
        // self.function_id.hash(state);
    }
}
