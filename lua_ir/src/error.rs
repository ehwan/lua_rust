use crate::{LuaEnv, LuaValue};

// @TODO
// error should match with (real) lua error
#[derive(Debug, Clone)]
pub enum RuntimeError {
    InvalidArith,
    ExpectedMultire,
    FunctionCallOnNonFunction,
    TypeError,
    /// table index is nil
    TableIndexNil,
    /// table index is NaN
    TableIndexNan,
    OutOfRange,
    /// float has no integer representation
    FloatToInt,
    NotInteger,
    NotTable,
    NotString,
    NotNumber,
    NotFunction,
    NotThread,

    /// resume, yield, close called on non-coroutine
    NotCoroutine,

    ThreadDead,

    NoMetaMethod,

    /// value expected but not passed ( not enough arguments )
    ValueExpected,

    /// rawlen
    NotTableOrString,

    /// try to modify protected metatable (__metatable defined)
    ProtectedMetatable,

    /// string.char()
    OutOfRangeChar,

    /// error with error handler
    Error,

    /// next() called with invalid key
    InvalidKey,

    YieldOnMain,

    ResumeOnRunning,
    ResumeOnDead,
    ResumeOnParent,

    CloseCurrentThread,
    CloseParentThread,

    Custom(LuaValue),
}

impl RuntimeError {
    pub fn into_lua_value(self, _env: &mut LuaEnv) -> LuaValue {
        let string = format!("{:?}", self);
        LuaValue::String(string.into_bytes())
    }
}
