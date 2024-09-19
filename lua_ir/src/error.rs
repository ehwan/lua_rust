#[derive(Debug)]
pub enum RuntimeError {
    InvalidArith,
    ExpectedMultire,
    FunctionCallOnNonFunction,
    TypeError,
}
