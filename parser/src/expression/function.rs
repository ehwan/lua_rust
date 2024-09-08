use crate::statement;

/// parameter list for named & anonymous function definition
#[derive(Clone, Debug)]
pub struct ParameterList {
    pub names: Vec<String>,
    /// is `...` present?
    pub variadic: bool,
}
impl ParameterList {
    pub fn new(names: Vec<String>, variadic: bool) -> Self {
        Self { names, variadic }
    }
}

/// unnamed function
#[derive(Clone, Debug)]
pub struct ExprFunction {
    /// function parameters
    pub parameters: ParameterList,
    /// function body to be executed
    pub block: statement::Block,
}
impl ExprFunction {
    pub fn new(parameters: Option<ParameterList>, block: statement::Block) -> Self {
        if let Some(p) = parameters {
            Self {
                parameters: p,
                block,
            }
        } else {
            Self {
                parameters: ParameterList::new(Vec::new(), false),
                block,
            }
        }
    }
}
