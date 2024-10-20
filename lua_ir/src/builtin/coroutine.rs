use std::rc::Rc;

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
    coroutine.map.insert(
        "resume".into(),
        LuaFunction::from_func_with_expected(resume).into(),
    );
    coroutine
        .map
        .insert("running".into(), LuaFunction::from_func(running).into());
    coroutine
        .map
        .insert("status".into(), LuaFunction::from_func(status).into());
    coroutine
        .map
        .insert("wrap".into(), LuaFunction::from_func(wrap).into());
    coroutine.map.insert(
        "yield".into(),
        LuaFunction::from_func_with_expected(yield_).into(),
    );
    Ok(coroutine.into())
}

pub fn close(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    env.pop_n(args - 1);
    let co = match env.pop() {
        LuaValue::Thread(thread) => thread,
        _ => return Err(RuntimeError::NotThread),
    };

    let status = co.borrow().status;
    match status {
        ThreadStatus::Running => Err(RuntimeError::CloseCurrentThread),
        ThreadStatus::ResumePending(_) => Err(RuntimeError::CloseParentThread),

        ThreadStatus::NotStarted => {
            env.push(true.into());
            Ok(1)
        }
        // Dead, YieldPending
        _ => {
            // @TODO
            // check error
            env.push(true.into());
            Ok(1)
        }
    }
}
pub fn create(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    env.pop_n(args - 1);
    let func = env.pop();
    match func {
        LuaValue::Function(func) => {
            env.push(LuaThread::new_coroutine(env, func).into());
            Ok(1)
        }
        _ => Err(RuntimeError::NotFunction),
    }
}
pub fn isyieldable(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    // thread is yieldable if it is not the main thread
    let is_yieldable = match args {
        0 => !Rc::ptr_eq(env.main_thread(), env.running_thread()),
        _ => {
            env.pop_n(args - 1);
            let thread = env.pop();
            match thread {
                LuaValue::Thread(thread) => !Rc::ptr_eq(env.main_thread(), &thread),
                _ => return Err(RuntimeError::NotThread),
            }
        }
    };
    env.push(is_yieldable.into());
    Ok(1)
}

pub fn resume(
    env: &mut LuaEnv,
    args_num: usize,
    expected_resume_return: Option<usize>,
) -> Result<(), RuntimeError> {
    if args_num == 0 {
        return Err(RuntimeError::ValueExpected);
    }

    debug_assert!(env.running_thread().borrow().status == ThreadStatus::Running);

    let co = match env.top_i(args_num - 1) {
        LuaValue::Thread(thread) => thread,
        _ => {
            env.pop_n(args_num);
            return Err(RuntimeError::NotThread);
        }
    };
    let mut co_borrow_mut = co.borrow_mut();
    // @TODO error message and return false
    match co_borrow_mut.status {
        ThreadStatus::Running => {
            env.pop_n(args_num);
            Err(RuntimeError::ResumeOnRunning)
        }
        ThreadStatus::Dead => {
            env.pop_n(args_num);
            Err(RuntimeError::ResumeOnDead)
        }
        ThreadStatus::ResumePending(_) => {
            env.pop_n(args_num);
            Err(RuntimeError::ResumeOnParent)
        }
        ThreadStatus::NotStarted => {
            let func = co_borrow_mut.func.clone().unwrap();

            env.borrow_running_thread_mut().status =
                ThreadStatus::ResumePending(expected_resume_return);
            co_borrow_mut.status = ThreadStatus::Running;
            co_borrow_mut
                .data_stack
                .extend(env.borrow_running_thread_mut().drain_last(args_num - 1));
            env.pop();
            drop(co_borrow_mut);
            env.coroutines.push(co);

            let func_borrow = func.borrow();
            match &*func_borrow {
                LuaFunction::LuaFunc(lua_func) => {
                    let upvalues = lua_func.upvalues.clone();
                    let func_id = lua_func.function_id;
                    // no need to pass `expected_ret`, since it was passed by `ThreadStatus::ResumePending(_)`
                    env.function_call_lua(args_num - 1, upvalues, func_id, None)
                }
                LuaFunction::RustFunc(rust_func) => {
                    let rust_func_ret = match expected_resume_return {
                        Some(0) => rust_func(env, args_num - 1, Some(0)),
                        Some(expected_resume_return) => {
                            rust_func(env, args_num - 1, Some(expected_resume_return - 1))
                        }
                        None => rust_func(env, args_num - 1, None),
                    };
                    match rust_func_ret {
                        Ok(_) => {
                            let mut co_borrow_mut = env.borrow_running_thread_mut();
                            co_borrow_mut.set_dead();
                            drop(co_borrow_mut);
                            env.coroutines.pop();
                            env.borrow_running_thread_mut().status = ThreadStatus::Running;
                            Ok(())
                        }
                        Err(_) => {
                            unimplemented!("coroutine.resume: error handling");
                        }
                    }
                }
            }
        }
        ThreadStatus::YieldPending(expected_yield_return) => {
            co_borrow_mut.status = ThreadStatus::Running;
            co_borrow_mut
                .data_stack
                .extend(env.borrow_running_thread_mut().drain_last(args_num - 1));
            env.pop();
            if let Some(expected_yield_return) = expected_yield_return {
                let adjusted =
                    co_borrow_mut.data_stack.len() - (args_num - 1) + expected_yield_return;
                co_borrow_mut
                    .data_stack
                    .resize_with(adjusted, Default::default);
            }
            drop(co_borrow_mut);

            env.running_thread().borrow_mut().status =
                ThreadStatus::ResumePending(expected_resume_return);
            env.coroutines.push(co);

            Ok(())
        }
    }
}
pub fn yield_(
    env: &mut LuaEnv,
    args: usize,
    expected_yield_return: Option<usize>,
) -> Result<(), RuntimeError> {
    if env.coroutines.len() == 1 {
        env.pop_n(args);
        return Err(RuntimeError::YieldOnMain);
    }

    let yield_thread = env.coroutines.pop().unwrap();
    let mut yield_thread = yield_thread.borrow_mut();
    debug_assert_eq!(yield_thread.status, ThreadStatus::Running);
    yield_thread.status = ThreadStatus::YieldPending(expected_yield_return);

    // @TODO check if error occurs; if so, return false
    let mut resume_thread = env.running_thread().borrow_mut();
    if let ThreadStatus::ResumePending(expected_resume_return) = resume_thread.status {
        match expected_resume_return {
            Some(0) => { /* do nothing */ }
            Some(expected_resume_return) => {
                resume_thread.data_stack.push(true.into());
                resume_thread
                    .data_stack
                    .extend(yield_thread.drain_last(args));
                let adjusted = resume_thread.data_stack.len() - args - 1 + expected_resume_return;
                resume_thread
                    .data_stack
                    .resize_with(adjusted, Default::default);
            }
            None => {
                resume_thread.data_stack.push(true.into());
                resume_thread
                    .data_stack
                    .extend(yield_thread.drain_last(args));
            }
        }
    } else {
        unreachable!("yield: resume thread status is not ResumePending");
    }
    resume_thread.status = ThreadStatus::Running;

    Ok(())
}

pub fn running(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    env.pop_n(args);
    let thread = Rc::clone(env.running_thread());
    let is_main = Rc::ptr_eq(&thread, env.main_thread());
    env.push2(LuaValue::Thread(thread), is_main.into());
    Ok(2)
}
pub fn status(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    if args == 0 {
        return Err(RuntimeError::ValueExpected);
    }
    env.pop_n(args - 1);
    let co = env.pop();
    let co = match co {
        LuaValue::Thread(thread) => thread,
        _ => return Err(RuntimeError::NotThread),
    };
    let status: &'static str = match co.borrow().status {
        ThreadStatus::NotStarted => "suspended",
        ThreadStatus::Running => "running",
        ThreadStatus::Dead => "dead",
        ThreadStatus::YieldPending(_) => "suspended",
        ThreadStatus::ResumePending(_) => "normal",
    };
    env.push(status.into());
    Ok(1)
}

// fn wrap_inner(
//     env: &mut LuaEnv,
//     thread: &Rc<RefCell<LuaThread>>,
//     chunk: &Chunk,
//     args: usize,
// ) -> Result<usize, RuntimeError> {
// }
pub fn wrap(env: &mut LuaEnv, args: usize) -> Result<usize, RuntimeError> {
    create(env, args)?;
    let co = env.pop();
    let _co = match co {
        LuaValue::Thread(thread) => thread,
        _ => return Err(RuntimeError::NotThread),
    };

    unimplemented!("coroutine.wrap")
}
