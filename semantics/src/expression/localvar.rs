#[derive(Clone, Debug)]
pub enum ExprLocalVariable {
    /// on stack
    Stack(usize, String),
    /// on upvalue
    Upvalue(usize, String),
}
