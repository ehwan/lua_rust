/// Goto statement
#[derive(Clone, Debug)]
pub struct StmtGoto {
    pub name: String,
}
impl StmtGoto {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
