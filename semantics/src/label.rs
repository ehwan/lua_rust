use lua_parser::{Span, SpannedString};

use crate::ProcessError;

/// label information.
#[derive(Debug)]
pub struct LabelInfo {
    pub name: String,

    /// Scope tree of this label.
    /// In goto statement, destination label must be parent (prefix) of current scope.
    pub scope: Option<(Vec<usize>, Span)>,

    /// where `goto` statement is called
    pub from: Vec<(Vec<usize>, Span)>,
}

impl LabelInfo {
    fn check_prefix(prefix: &[usize], target: &[usize]) -> bool {
        if prefix.len() > target.len() {
            return false;
        }
        for i in 0..prefix.len() {
            if prefix[i] != target[i] {
                return false;
            }
        }
        true
    }
    pub fn add_from(&mut self, from: Vec<usize>, goto_span: Span) -> Result<(), ProcessError> {
        if let Some((scope, label_span)) = &self.scope {
            if !Self::check_prefix(scope, &from) {
                return Err(ProcessError::InvalidGotoScope(*label_span, goto_span));
            }
        }
        self.from.push((from, goto_span));
        Ok(())
    }
    pub fn set_label(
        &mut self,
        name: SpannedString,
        scope: Vec<usize>,
        label_span: Span,
    ) -> Result<(), ProcessError> {
        if self.scope.is_some() {
            Err(ProcessError::MultipleLabel(name))
        } else {
            for (from, from_span) in &self.from {
                if !Self::check_prefix(&scope, from) {
                    return Err(ProcessError::InvalidGotoScope(label_span, *from_span));
                }
            }

            self.name = name.string;
            self.scope = Some((scope, label_span));
            Ok(())
        }
    }
}

impl Default for LabelInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            scope: None,
            from: Vec::new(),
        }
    }
}
