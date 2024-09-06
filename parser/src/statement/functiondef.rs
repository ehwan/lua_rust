/// function name.
/// a sequence of identifiers separated by dots, and an optional colon followed by an identifier.
/// e.g. `a.b.c:d`
#[derive(Clone, Debug)]
pub struct FunctionName {
    /// dot chain
    pub names: Vec<String>,
    /// colon chain at the end
    pub colon: Option<String>,
}
impl FunctionName {
    pub fn new(names: Vec<String>, colon: Option<String>) -> Self {
        Self { names, colon }
    }
}

/// function definition statement.
#[derive(Clone, Debug)]
pub struct StmtFunctionDefinition {
    pub name: FunctionName,
    pub body: crate::expression::ExprFunction,
}
impl StmtFunctionDefinition {
    pub fn new(name: FunctionName, body: crate::expression::ExprFunction) -> Self {
        Self { name, body }
    }
}

/// local function definition statement.
#[derive(Clone, Debug)]
pub struct StmtFunctionDefinitionLocal {
    pub name: String,
    pub body: crate::expression::ExprFunction,
}
impl StmtFunctionDefinitionLocal {
    pub fn new(name: String, body: crate::expression::ExprFunction) -> Self {
        Self { name, body }
    }
}
