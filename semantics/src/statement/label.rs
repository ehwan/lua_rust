use crate::LabelInfo;
use std::cell::RefCell;
use std::rc::Rc;

/// Goto statement
#[derive(Clone, Debug)]
pub struct StmtLabel {
    pub label: Rc<RefCell<LabelInfo>>,
}
impl StmtLabel {
    pub fn new(label: Rc<RefCell<LabelInfo>>) -> Self {
        Self { label }
    }
}
