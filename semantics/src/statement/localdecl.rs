use std::{cell::RefCell, rc::Rc};

pub use lua_parser::Attrib;

use crate::{Expression, VariableInfo};

/// local variable declaration.
#[derive(Clone, Debug)]
pub struct StmtLocalDeclaration {
    /// (stack offset, attribute)
    pub decls: Vec<(Rc<RefCell<VariableInfo>>, Option<Attrib>)>,
    pub values: Option<Vec<Expression>>,
}
impl StmtLocalDeclaration {
    pub fn new(
        decls: Vec<(Rc<RefCell<VariableInfo>>, Option<Attrib>)>,
        values: Option<Vec<Expression>>,
    ) -> Self {
        Self { decls, values }
    }
}
