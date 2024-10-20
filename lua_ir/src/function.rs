use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;

use crate::LuaEnv;
use crate::LuaValue;
use crate::RuntimeError;

/// function information
#[derive(Debug, Clone, Copy)]
pub struct FunctionInfo {
    /// number of arguments ( excluding variadic arguments )
    pub args: usize,
    /// if true, this function is variadic
    pub is_variadic: bool,
    /// number of local variables for this function (including arguments)
    pub local_variables: usize,
    /// entry point of this function ( instruction index )
    pub address: usize,
}

/// built-in functions written in Rust
type LuaFunctionRust = Box<dyn Fn(&mut LuaEnv, usize, Option<usize>) -> Result<(), RuntimeError>>;

/// lua function object
pub enum LuaFunction {
    /// functions written in Lua
    LuaFunc(LuaFunctionLua),
    /// built-in functions written in Rust
    RustFunc(LuaFunctionRust),
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
    pub fn from_func_with_expected(
        func: impl Fn(&mut LuaEnv, usize, Option<usize>) -> Result<(), RuntimeError> + 'static,
    ) -> Self {
        LuaFunction::RustFunc(Box::new(func))
    }
    pub fn from_func(
        func: impl Fn(&mut LuaEnv, usize) -> Result<usize, RuntimeError> + 'static,
    ) -> Self {
        Self::from_func_with_expected(move |env, stack_top, expected| {
            let ret = func(env, stack_top)?;
            if let Some(expected) = expected {
                let mut thread_mut = env.borrow_running_thread_mut();
                let adjusted = thread_mut.data_stack.len() - ret + expected;
                thread_mut
                    .data_stack
                    .resize_with(adjusted, Default::default);
            }
            Ok(())
        })
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
