use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::LuaEnv;
use crate::LuaFunction;
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
    // if args == 0 {
    //     return Err(RuntimeError::ValueExpected);
    // }
    // let co = thread.borrow_mut().pop1(args);
    // match co {
    //     LuaValue::Thread(thread) => {}
    //     _ => Err(RuntimeError::NotThread),
    // }

    unimplemented!("coroutine.close")
}
pub fn create(
    _env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    // if args == 0 {
    //     return Err(RuntimeError::ValueExpected);
    // }
    // let func = thread.borrow_mut().pop1(args);
    // match func {
    //     LuaValue::Function(func) => {
    //         let thread = Rc::new(RefCell::new(LuaThread::new()));
    //         let mut thread_mut = thread.borrow_mut();
    //         thread_mut.data_stack.push(func.into());
    //         Ok(1)
    //     }
    //     _ => Err(RuntimeError::NotFunction),
    // }
    unimplemented!("coroutine.create")
}
pub fn isyieldable(
    env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    // let co = match args {
    //     0 => thread.clone(),
    //     _ => match thread.borrow_mut().pop1(args) {
    //         LuaValue::Thread(thread) => thread,
    //         _ => return Err(RuntimeError::NotThread),
    //     },
    // };

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
    env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    let mut thread_mut = thread.borrow_mut();
    drop(thread_mut.pop_n(args));
    thread_mut
        .data_stack
        .push(LuaValue::Thread(Rc::clone(thread)));
    thread_mut
        .data_stack
        .push(Rc::ptr_eq(thread, env.main_thread()).into());
    Ok(2)
}
pub fn status(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    // if args == 0 {
    //     return Err(RuntimeError::ValueExpected);
    // }
    // let co = thread.borrow_mut().pop1(args);
    // match co {
    //     LuaValue::Thread(thread) => {}
    //     _ => Err(RuntimeError::NotThread),
    // }
    unimplemented!("coroutine.status")
}
pub fn wrap(
    _env: &mut LuaEnv,
    _thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    _args: usize,
) -> Result<usize, RuntimeError> {
    // if args == 0 {
    //     return Err(RuntimeError::ValueExpected);
    // }
    // let func = thread.borrow_mut().pop1(args);
    // match func {
    //     LuaValue::Function(func) => {
    //         let thread = Rc::new(RefCell::new(LuaThread::new()));
    //         let mut thread_mut = thread.borrow_mut();
    //         thread_mut.data_stack.push(func.into());
    //         Ok(1)
    //     }
    //     _ => Err(RuntimeError::NotFunction),
    // }
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
