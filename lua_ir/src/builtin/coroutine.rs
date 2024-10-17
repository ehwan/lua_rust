use std::cell::RefCell;
use std::rc::Rc;

use crate::Chunk;
use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaThread;
use crate::LuaValue;
use crate::RuntimeError;
use crate::ThreadStatus;

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
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    let func = thread.borrow_mut().pop1(args);
    match func {
        LuaValue::Function(func) => {
            let mut new_thread = LuaThread::new();
            new_thread.status = Some(crate::vm::ThreadStatus::Init);
            new_thread.func = Some(func);
            thread.borrow_mut().data_stack.push(new_thread.into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotFunction),
    }
}
pub fn isyieldable(
    env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    _chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    let co = match args {
        0 => Rc::clone(thread),
        _ => match thread.borrow_mut().pop1(args) {
            LuaValue::Thread(thread) => thread,
            _ => return Err(RuntimeError::NotThread),
        },
    };

    // thread is yieldable if it is not the main thread
    thread
        .borrow_mut()
        .data_stack
        .push((!Rc::ptr_eq(&co, env.main_thread())).into());
    Ok(1)
}

pub fn resume(
    env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    chunk: &Chunk,
    args_num: usize,
) -> Result<usize, RuntimeError> {
    if args_num == 0 {
        return Err(RuntimeError::ValueExpected);
    }

    /*
    let mut thread_mut = thread.borrow_mut();
    let mut it = thread_mut.pop_n(args_num);
    let co = it.next().unwrap();
    let mut args: Vec<LuaValue> = it.collect();
    drop(it);
    drop(thread_mut);
    match co {
        LuaValue::Thread(resume_thread) => {
            let status = resume_thread.borrow().status;

            match status {
                None => {
                    return Err(RuntimeError::NotCoroutine);
                }
                Some(ThreadStatus::Init) => {
                    resume_thread.borrow_mut().data_stack.append(&mut args);
                    let func = Rc::clone(resume_thread.borrow().func.as_ref().unwrap());
                    env.function_call(
                        &resume_thread,
                        chunk,
                        args_num - 1,
                        LuaValue::Function(func),
                        expected_ret,
                    )?;
                    Ok(1)
                }
                Some(ThreadStatus::Running) => {}
                Some(ThreadStatus::Dead) => {
                    return Err(RuntimeError::ThreadDead);
                }
            }
        }
        _ => Err(RuntimeError::NotThread),
    }
    */
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

// fn wrap_inner(
//     env: &mut LuaEnv,
//     thread: &Rc<RefCell<LuaThread>>,
//     chunk: &Chunk,
//     args: usize,
// ) -> Result<usize, RuntimeError> {
// }
pub fn wrap(
    env: &mut LuaEnv,
    thread: &Rc<RefCell<LuaThread>>,
    chunk: &Chunk,
    args: usize,
) -> Result<usize, RuntimeError> {
    // create(env, thread, chunk, args)?;
    // let co = thread.borrow_mut().pop1(1);

    // let func = thread.borrow_mut().pop1(args);
    // match func {
    //     LuaValue::Function(func) => {}
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
