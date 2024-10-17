use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaNumber;
use crate::LuaTable;
use crate::LuaThread;
use crate::LuaValue;
use crate::RuntimeError;

/// init coroutine module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut coroutine = LuaTable::new();
    coroutine
        .map
        .insert("close".into(), LuaFunction::from_func(close).into());
    coroutine
        .map
        .insert("create".into(), LuaFunction::from_func(create).into());
    coroutine.map.insert(
        "isyieldable".into(),
        LuaFunction::from_func(isyieldable).into(),
    );
    coroutine
        .map
        .insert("resume".into(), LuaFunction::from_func(resume).into());
    coroutine
        .map
        .insert("running".into(), LuaFunction::from_func(running).into());
    coroutine
        .map
        .insert("status".into(), LuaFunction::from_func(status).into());
    coroutine
        .map
        .insert("wrap".into(), LuaFunction::from_func(wrap).into());
    coroutine
        .map
        .insert("yield".into(), LuaFunction::from_func(yield_).into());
    Ok(coroutine.into())
}

pub fn close(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.close")
}
pub fn create(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.create")
}
pub fn isyieldable(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.isyieldable")
}

pub fn resume(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.resume")
}

pub fn running(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.status")
}
pub fn status(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.status")
}
pub fn wrap(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.wrap")
}
pub fn yield_(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    unimplemented!("coroutine.yield")
}
