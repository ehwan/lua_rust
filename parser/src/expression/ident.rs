/// just identifier
#[derive(Clone, Debug)]
pub struct ExprIdent {
    pub name: String,
}
impl ExprIdent {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
