use lua_tokenizer::TokenizeError;

use crate::LuaString;
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

    AttemptToGetLengthOf(&'static str),
    AttemptToArithmeticOn(&'static str),
    AttemptToBitwiseOn(&'static str),
    AttemptToConcatenate(&'static str),

    /// when converting a float to int, the float has a fractional part
    NoIntegerRepresentation,

    TokenizeError(TokenizeError),

    /// tonumber
    BaseOutOfRange,

    /// string coherence arithmetic
    /// method, lhs, rhs
    AttemptTo(&'static str, &'static str, &'static str),

    // ========================
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
                LuaValue::String(LuaString::from_string(string))
            }
        }
    }
    pub fn to_error_message(&self, env: &LuaEnv) -> String {
        RuntimeErrorEnvPair(&self, env).to_string()
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
            RuntimeError::AttemptToGetLengthOf(type_str) => {
                write!(f, "attempt to get length of a {} value", type_str)
            }
            RuntimeError::AttemptToArithmeticOn(type_str) => {
                write!(f, "attempt to perform arithmetic on a {} value", type_str)
            }
            RuntimeError::AttemptToBitwiseOn(type_str) => {
                write!(
                    f,
                    "attempt to perform bitwise operation on a {} value",
                    type_str
                )
            }
            RuntimeError::AttemptToConcatenate(type_str) => {
                write!(f, "attempt to concatenate a {} value", type_str)
            }
            RuntimeError::NoIntegerRepresentation => "number has no integer representation".fmt(f),
            RuntimeError::TokenizeError(err) => write!(f, "{}", err),
            RuntimeError::BaseOutOfRange => "base out of range".fmt(f),
            RuntimeError::AttemptTo(method, lhs, rhs) => {
                write!(f, "attempt to {} a '{}' with a '{}'", method, lhs, rhs)
            }

            RuntimeError::Custom(val) => write!(f, "{}", val),
            _ => write!(f, "{:?}", self.0),
        }
    }
}
