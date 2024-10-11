#[derive(Debug)]
pub enum RuntimeError {
    InvalidArith,
    ExpectedMultire,
    FunctionCallOnNonFunction,
    TypeError,
    GetOnNonTable,
    SetOnNonTable,
    /// table index is nil
    TableIndexNil,
    /// table index is NaN
    TableIndexNan,
    /// float has no integer representation
    FloatToInt,
    NotInteger,
    OutOfRange,

    NotString,

    NotNumber,
}
