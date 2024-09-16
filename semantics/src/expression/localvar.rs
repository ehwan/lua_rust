#[derive(Clone, Debug)]
pub enum ExprLocalVariable {
    /// on stack
    Stack(usize),
    /// on upvalue
    Upvalue(usize),
}
