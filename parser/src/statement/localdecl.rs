use crate::ExprString;
use crate::Expression;
use crate::Span;

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
    pub name: ExprString,
    pub attrib: Option<Attrib>,
    pub span: Span,
}
impl AttName {
    pub fn new(name: ExprString, attrib: Option<Attrib>, span: Span) -> Self {
        Self { name, attrib, span }
    }
    /// get the span of the whole variable name and attribute.
    pub fn span(&self) -> Span {
        self.span
    }
}

/// local variable declaration.
#[derive(Clone, Debug)]
pub struct StmtLocalDeclaration {
    pub names: Vec<AttName>,
    /// `Some` if the variables are initialized.
    pub values: Option<Vec<Expression>>,

    pub span: Span,
}
impl StmtLocalDeclaration {
    pub fn new(names: Vec<AttName>, values: Option<Vec<Expression>>, span: Span) -> Self {
        Self {
            names,
            values,
            span,
        }
    }
    /// get the span of the whole local variable declaration.
    pub fn span(&self) -> Span {
        self.span
    }
}
