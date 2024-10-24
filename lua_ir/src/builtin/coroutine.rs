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
        return Err(RuntimeError::new_empty_argument(1, "thread"));
    }
    env.pop_n(args - 1);
    let co = env.pop();
    let co = match co {
        LuaValue::Thread(thread) => thread,
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("thread", co.type_str().into())),
            ))
        }
    };

    let status = co.borrow().status;
    match status {
        ThreadStatus::Running => Err(RuntimeError::CloseRunningThread),
        ThreadStatus::ResumePending(_) => Err(RuntimeError::CloseNormalThread),

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
        return Err(RuntimeError::new_empty_argument(1, "thread"));
    }
    env.pop_n(args - 1);
    let func = env.pop();
    match func {
        LuaValue::Function(func) => {
            env.push(LuaThread::new_coroutine(env, func).into());
            Ok(1)
        }
        _ => Err(RuntimeError::BadArgument(
            1,
            Box::new(RuntimeError::Expected("function", func.type_str().into())),
        )),
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
                _ => {
                    return Err(RuntimeError::BadArgument(
                        1,
                        Box::new(RuntimeError::Expected("thread", thread.type_str().into())),
                    ))
                }
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
        return Err(RuntimeError::new_empty_argument(1, "thread"));
    }

    debug_assert!(env.running_thread().borrow().status == ThreadStatus::Running);

    let co = env.top_i(args_num - 1);
    let co = match co {
        LuaValue::Thread(thread) => thread,
        _ => {
            env.pop_n(args_num);
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("thread", co.type_str().into())),
            ));
        }
    };
    let mut co_borrow_mut = co.borrow_mut();
    match co_borrow_mut.status {
        ThreadStatus::Running => {
            env.pop_n(args_num);
            // error resume() into current thread
            match expected_resume_return {
                Some(0) => Ok(()),
                Some(1) => {
                    env.push(false.into());
                    Ok(())
                }
                Some(expected_resume_return) => {
                    env.push2(false.into(), "cannot resume running coroutine".into());
                    env.running_thread()
                        .borrow_mut()
                        .data_stack
                        .extend(std::iter::repeat(LuaValue::Nil).take(expected_resume_return - 2));
                    Ok(())
                }
                None => {
                    env.push2(false.into(), "cannot resume running coroutine".into());
                    Ok(())
                }
            }
        }
        ThreadStatus::Dead => {
            env.pop_n(args_num);
            // error resume() into dead thread
            match expected_resume_return {
                Some(0) => Ok(()),
                Some(1) => {
                    env.push(false.into());
                    Ok(())
                }
                Some(expected_resume_return) => {
                    env.push2(false.into(), "cannot resume dead coroutine".into());
                    env.running_thread()
                        .borrow_mut()
                        .data_stack
                        .extend(std::iter::repeat(LuaValue::Nil).take(expected_resume_return - 2));
                    Ok(())
                }
                None => {
                    env.push2(false.into(), "cannot resume dead coroutine".into());
                    Ok(())
                }
            }
        }
        ThreadStatus::ResumePending(_) => {
            env.pop_n(args_num);

            // error resume() into parent thread
            match expected_resume_return {
                Some(0) => Ok(()),
                Some(1) => {
                    env.push(false.into());
                    Ok(())
                }
                Some(expected_resume_return) => {
                    env.push2(false.into(), "cannot resume non-suspended coroutine".into());
                    env.running_thread()
                        .borrow_mut()
                        .data_stack
                        .extend(std::iter::repeat(LuaValue::Nil).take(expected_resume_return - 2));
                    Ok(())
                }
                None => {
                    env.push2(false.into(), "cannot resume non-suspended coroutine".into());
                    Ok(())
                }
            }
        }
        ThreadStatus::NotStarted => {
            env.borrow_running_thread_mut().status =
                ThreadStatus::ResumePending(expected_resume_return);
            co_borrow_mut.status = ThreadStatus::Running;
            co_borrow_mut
                .data_stack
                .extend(env.borrow_running_thread_mut().drain_last(args_num - 1));
            // pop `co`
            env.pop();
            drop(co_borrow_mut);
            let func = Rc::clone(co.borrow().function.as_ref().unwrap());
            env.coroutines.push(co);
            let function_call_res = env.function_call(
                args_num - 1,
                LuaValue::Function(func),
                expected_resume_return,
            );
            match function_call_res {
                Ok(_) => {}

                Err(err) => {
                    let error_object = err.into_lua_value(env);
                    env.coroutines.pop().unwrap().borrow_mut().set_dead();
                    env.running_thread().borrow_mut().status = ThreadStatus::Running;

                    match expected_resume_return {
                        Some(0) => {}
                        Some(1) => {
                            env.push(false.into());
                        }
                        Some(expected_resume_return) => {
                            env.push2(false.into(), error_object);
                            env.running_thread().borrow_mut().data_stack.extend(
                                std::iter::repeat(LuaValue::Nil).take(expected_resume_return - 2),
                            );
                        }
                        None => {
                            env.push2(false.into(), error_object);
                        }
                    }
                }
            }
            Ok(())
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
            let coroutine_len = env.coroutines.len();
            while env.coroutines.len() >= coroutine_len {
                let cycle_res = env.cycle();
                match cycle_res {
                    Ok(_) => {}
                    Err(err) => {
                        let error_object = err.into_lua_value(env);
                        env.coroutines.pop().unwrap().borrow_mut().set_dead();
                        env.running_thread().borrow_mut().status = ThreadStatus::Running;

                        match expected_resume_return {
                            Some(0) => {}
                            Some(1) => {
                                env.push(false.into());
                            }
                            Some(expected_resume_return) => {
                                env.push2(false.into(), error_object);
                                env.running_thread().borrow_mut().data_stack.extend(
                                    std::iter::repeat(LuaValue::Nil)
                                        .take(expected_resume_return - 2),
                                );
                            }
                            None => {
                                env.push2(false.into(), error_object);
                            }
                        }
                    }
                }
            }

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
        return Err(RuntimeError::YieldOutsideCoroutine);
    }

    let yield_thread = env.coroutines.pop().unwrap();
    let mut yield_thread = yield_thread.borrow_mut();
    debug_assert_eq!(yield_thread.status, ThreadStatus::Running);
    yield_thread.status = ThreadStatus::YieldPending(expected_yield_return);

    let mut resume_thread = env.running_thread().borrow_mut();
    if let ThreadStatus::ResumePending(expected_resume_return) = resume_thread.status {
        match expected_resume_return {
            Some(0) => {
                let trunc_len = yield_thread.data_stack.len() - args;
                yield_thread.data_stack.truncate(trunc_len);
            }
            Some(1) => {
                resume_thread.data_stack.push(true.into());
                let trunc_len = yield_thread.data_stack.len() - args;
                yield_thread.data_stack.truncate(trunc_len);
            }
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
        return Err(RuntimeError::new_empty_argument(1, "thread"));
    }
    env.pop_n(args - 1);
    let co = env.pop();
    let co = match co {
        LuaValue::Thread(thread) => thread,
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("thread", co.type_str().into())),
            ))
        }
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
        _ => {
            return Err(RuntimeError::BadArgument(
                1,
                Box::new(RuntimeError::Expected("thread", co.type_str().into())),
            ))
        }
    };

    unimplemented!("coroutine.wrap")
}
