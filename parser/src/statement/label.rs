/// label definition
#[derive(Clone, Debug)]
pub struct StmtLabel {
    pub name: String,
}
impl StmtLabel {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
