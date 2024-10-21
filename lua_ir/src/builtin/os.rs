use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;

/// init os module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut os = LuaTable::new();

    os.insert("clock".into(), LuaFunction::from_func(clock).into());

    os.insert("date".into(), LuaFunction::from_func(date).into());
    os.insert("difftime".into(), LuaFunction::from_func(difftime).into());
    os.insert("execute".into(), LuaFunction::from_func(execute).into());
    os.insert("exit".into(), LuaFunction::from_func(exit).into());
    os.insert("getenv".into(), LuaFunction::from_func(getenv).into());
    os.insert("remove".into(), LuaFunction::from_func(remove).into());
    os.insert("rename".into(), LuaFunction::from_func(rename).into());
    os.insert("setlocale".into(), LuaFunction::from_func(setlocale).into());
    os.insert("time".into(), LuaFunction::from_func(time).into());
    os.insert("tmpname".into(), LuaFunction::from_func(tmpname).into());

    Ok(os.into())
}

pub fn clock(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.clock");
}
pub fn date(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.date");
}

pub fn difftime(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.difftime");
}

pub fn execute(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.execute");
}

pub fn exit(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.exit");
}

pub fn getenv(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.getenv");
}

pub fn remove(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.remove");
}

pub fn rename(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.rename");
}

pub fn setlocale(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.setlocale");
}

pub fn time(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.time");
}

pub fn tmpname(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("os.tmpname");
}
