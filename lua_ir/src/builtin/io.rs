use crate::LuaEnv;
use crate::LuaFunction;
use crate::LuaTable;
use crate::LuaValue;
use crate::RuntimeError;

/// init math module
pub fn init() -> Result<LuaValue, RuntimeError> {
    let mut io = LuaTable::new();
    io.insert("close".into(), LuaFunction::from_func(close).into());
    io.insert("flush".into(), LuaFunction::from_func(flush).into());
    io.insert("input".into(), LuaFunction::from_func(input).into());
    io.insert("lines".into(), LuaFunction::from_func(lines).into());
    io.insert("open".into(), LuaFunction::from_func(open).into());
    io.insert("output".into(), LuaFunction::from_func(output).into());
    io.insert("popen".into(), LuaFunction::from_func(popen).into());
    io.insert("read".into(), LuaFunction::from_func(read).into());
    io.insert("tmpfile".into(), LuaFunction::from_func(tmpfile).into());
    io.insert("type".into(), LuaFunction::from_func(type_).into());
    io.insert("write".into(), LuaFunction::from_func(write).into());
    Ok(io.into())
}

pub fn close(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.close");
}
pub fn flush(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.flush");
}
pub fn input(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.input");
}
pub fn lines(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.lines");
}
pub fn open(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.open");
}
pub fn output(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.output");
}
pub fn popen(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.popen");
}

pub fn read(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.read");
}
pub fn tmpfile(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.tmpfile");
}
pub fn type_(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.type");
}
pub fn write(_env: &mut LuaEnv, _args: usize) -> Result<usize, RuntimeError> {
    unimplemented!("io.write");
}
