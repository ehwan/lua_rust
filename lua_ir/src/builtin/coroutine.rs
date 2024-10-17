use std::sync::Arc;
use std::sync::RwLock;

use crate::Chunk;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaThread;
use crate::LuaValue;
use crate::RuntimeError;

/// init math module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut coroutine = LuaTable::new();

    coroutine.insert("close".into(), LuaFunction::from_func(close).into());
    coroutine.insert("create".into(), LuaFunction::from_func(create).into());
    coroutine.insert(
        "isyieldable".into(),
        LuaFunction::from_func(isyieldable).into(),
    );
    coroutine.insert("resume".into(), LuaFunction::from_func(resume).into());
    coroutine.insert("running".into(), LuaFunction::from_func(running).into());
    coroutine.insert("status".into(), LuaFunction::from_func(status).into());
    coroutine.insert("wrap".into(), LuaFunction::from_func(wrap).into());
    coroutine.insert("yield".into(), LuaFunction::from_func(yield_).into());
    Ok(coroutine.into())
}

pub fn close(
    _env: &mut LuaEnv,
    _thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.close")
}
pub fn create(
    _env: &mut LuaEnv,
    _thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.create")
    /*
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let func = match stack.pop1(args) {
        LuaValue::Function(f) => f,
        _ => return Err(RuntimeError::NotFunction),
    };
    let thread = LuaThread::from_func(func);
    let thread = Arc::new(RwLock::new(thread));
    stack.data_stack.push(LuaValue::Thread(thread));
    Ok(1)
    */
}

pub fn isyieldable(
    _env: &mut LuaEnv,
    _thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.isyieldable")
}

pub fn resume(
    _env: &mut LuaEnv,
    _thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.resume")
}

pub fn running(
    _env: &mut LuaEnv,
    _thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.running")
}

pub fn status(
    _env: &mut LuaEnv,
    _thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.status")
}
pub fn wrap(
    _env: &mut LuaEnv,
    _thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.wrap")
}

pub fn yield_(
    _env: &mut LuaEnv,
    _thread: &Arc<RwLock<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.yield")
}
