use crate::{LuaEnv, LuaValue};

// @TODO
// error should match with (real) lua error
#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// custom error object, or any unique-string error message from built-in functions
    Custom(LuaValue),

    /// error occured in function call, on argument at index `usize`
    BadArgument(usize, Box<RuntimeError>),

    /// rawset: table index is nil
    TableIndexNil,
    /// rawset: table index is NaN
    TableIndexNan,

    /// select
    IndexOutOfRange,

    /// table.insert, table.remove
    PositionOutOfBounds,

    /// A expected, got B
    Expected(&'static str, Option<&'static str>),

    /// string.char
    ValueOutOfRange,

    /// try to `coroutine.close()` a running(current) coroutine
    CloseRunningThread,
    /// try to `coroutine.close()` a normal(parent) coroutine
    CloseNormalThread,

    YieldOutsideCoroutine,

    // ========================
    /// float has no integer representation
    FloatToInt,
    NotInteger,
    NotNumber,

    NoMetaMethod,

    /// not implemented yet (dummy error for some functions)
    Error,
}

impl RuntimeError {
    /// create a new
    /// bad argument #idx to 'xxx' (`expected` expected, got no value)
    /// error.
    pub(crate) fn new_empty_argument(idx: usize, expected: &'static str) -> Self {
        RuntimeError::BadArgument(idx, Box::new(RuntimeError::Expected(expected, None)))
    }
    pub fn into_lua_value(self, env: &mut LuaEnv) -> LuaValue {
        match self {
            RuntimeError::Custom(val) => return val,
            _ => {
                let string = RuntimeErrorEnvPair(&self, env).to_string();
                LuaValue::String(string.into_bytes())
            }
        }
    }
}

struct RuntimeErrorEnvPair<'a>(&'a RuntimeError, &'a LuaEnv);

impl<'a> std::fmt::Display for RuntimeErrorEnvPair<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            RuntimeError::BadArgument(arg_idx, err) => write!(
                f,
                "bad argument #{} to '{}' ({})",
                arg_idx,
                self.1.last_op,
                RuntimeErrorEnvPair(err, self.1)
            ),
            RuntimeError::IndexOutOfRange => "index out of range".fmt(f),
            RuntimeError::PositionOutOfBounds => "position out of bounds".fmt(f),
            RuntimeError::TableIndexNil => "table index is nil".fmt(f),
            RuntimeError::TableIndexNan => "table index is NaN".fmt(f),
            RuntimeError::Expected(expected, got) => write!(
                f,
                "{} expected, got {}",
                expected,
                got.unwrap_or("no value")
            ),
            RuntimeError::ValueOutOfRange => "value out of range".fmt(f),
            RuntimeError::CloseRunningThread => "cannot close a running coroutine".fmt(f),
            RuntimeError::CloseNormalThread => "cannot close a normal coroutine".fmt(f),
            RuntimeError::YieldOutsideCoroutine => {
                "attempt to yield from outside a coroutine".fmt(f)
            }
            _ => write!(f, "{:?}", self.0),
        }
    }
}
