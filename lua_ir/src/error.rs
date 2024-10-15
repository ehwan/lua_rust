// @TODO
// error should match with (real) lua error
#[derive(Debug)]
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

    NoMetaMethod,

    /// value expected but not passed ( not enough arguments )
    ValueExpected,

    /// rawlen
    NotTableOrString,

    /// try to modify protected metatable (__metatable defined)
    ProtectedMetatable,

    /// string.char()
    OutOfRangeChar,

    /// table.concat
    NotStringOrNumber,

    /// error with error handler
    Error,
}
