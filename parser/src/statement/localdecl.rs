use crate::Expression;

/// local variable attribute.
/// either `const` or `close`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attrib {
    Const,
    Close,
}

/// pair of variable name and attribute.
#[derive(Clone, Debug)]
pub struct AttName {
    pub name: String,
    pub attrib: Option<Attrib>,
}
impl AttName {
    pub fn new(name: String, attrib: Option<Attrib>) -> Self {
        Self { name, attrib }
    }
}

/// local variable declaration.
#[derive(Clone, Debug)]
pub struct StmtLocalDeclaration {
    pub names: Vec<AttName>,
    pub values: Option<Vec<Expression>>,
}
impl StmtLocalDeclaration {
    pub fn new(names: Vec<AttName>, values: Option<Vec<Expression>>) -> Self {
        Self { names, values }
    }
}
