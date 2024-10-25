use std::{cell::RefCell, rc::Rc};

use crate::ExprLocalVariable;

#[derive(Debug, Clone)]
pub enum Scope {
    /// scope for local variable declaration.
    Block(ScopeBlock),
    /// scope for function
    Function(ScopeFunction),
}

#[derive(Debug, Clone)]
pub struct ScopeBlock {
    /// unique id for scope
    pub id: usize,

    /// To calculate stack size.
    pub max_variables: usize,

    /// stack offset of this scope
    pub offset: usize,

    /// variables in this scope, in order of declaration.
    pub variables: Vec<Rc<RefCell<VariableInfo>>>,

    pub is_loop: bool,

    pub labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// name of variable
    pub name: String,
    /// `true` if this variable should be reference. ( for upvalue capture )
    pub is_reference: bool,
    /// stack offset
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct UpvalueInfo {
    /// name of local variable
    pub name: String,
    /// where this upvalue is captured from, in parent function scope
    pub from: ExprLocalVariable,
}

#[derive(Debug, Clone)]
pub struct ScopeFunction {
    /// unique id for scope
    pub id: usize,

    /// To calculate stack size.
    pub max_variables: usize,

    pub upvalues: Vec<UpvalueInfo>,

    pub variadic: bool,
}
