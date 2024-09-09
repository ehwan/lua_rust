use crate::ExprString;
use crate::Span;

/// function name.
/// a sequence of identifiers separated by dots, and an optional colon followed by an identifier.
/// e.g. `a.b.c:d`
#[derive(Clone, Debug)]
pub struct FunctionName {
    /// dot chain
    pub names: Vec<ExprString>,
    /// colon chain at the end
    pub colon: Option<ExprString>,

    /// span of the whole function name
    pub span: Span,
}
impl FunctionName {
    pub fn new(names: Vec<ExprString>, colon: Option<ExprString>, span: Span) -> Self {
        Self { names, colon, span }
    }
    /// get the span of the whole function name
    pub fn span(&self) -> Span {
        self.span
    }
}

/// function definition statement.
#[derive(Clone, Debug)]
pub struct StmtFunctionDefinition {
    pub name: FunctionName,
    pub body: crate::expression::ExprFunction,
    pub span: Span,
}
impl StmtFunctionDefinition {
    pub fn new(name: FunctionName, body: crate::expression::ExprFunction, span: Span) -> Self {
        Self { name, body, span }
    }
    /// get the span of the whole function definition statement
    pub fn span(&self) -> Span {
        self.span
    }
}

/// local function definition statement.
#[derive(Clone, Debug)]
pub struct StmtFunctionDefinitionLocal {
    pub name: ExprString,
    pub body: crate::expression::ExprFunction,
    pub span: Span,
}
impl StmtFunctionDefinitionLocal {
    pub fn new(name: ExprString, body: crate::expression::ExprFunction, span: Span) -> Self {
        Self { name, body, span }
    }
    /// get the span of the whole local function definition statement
    pub fn span(&self) -> Span {
        self.span
    }
}
