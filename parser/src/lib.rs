//!
//! ```rust
//! let source = " <lua source code> ";
//!
//! let block = match lua_parser::parse_str(&source) {
//!     Ok(block) => block,
//!     Err(err) => {
//!         println!("{}", err);
//!         return;
//!     }
//! };
//!
//! println!("AST:\n{:#?}", block);
//! ```

mod error;
mod expression;
mod statement;
// mod parser;
mod parser_expanded;
use parser_expanded as parser;
mod spannedstring;

// re-exports
pub use spannedstring::SpannedString;

pub use lua_tokenizer::FloatType;
pub use lua_tokenizer::IntOrFloat;
pub use lua_tokenizer::IntType;
pub use lua_tokenizer::Span;
pub use lua_tokenizer::Token;

pub use error::InvalidToken;
pub use error::ParseError;
pub use error::TokenizeError;

pub use expression::ExprBinary;
pub use expression::ExprBinaryData;
pub use expression::ExprBool;
pub use expression::ExprFunction;
pub use expression::ExprFunctionCall;
pub use expression::ExprIdent;
pub use expression::ExprNil;
pub use expression::ExprNumeric;
pub use expression::ExprString;
pub use expression::ExprTable;
pub use expression::ExprTableIndex;
pub use expression::ExprUnary;
pub use expression::ExprUnaryData;
pub use expression::ExprVariadic;
pub use expression::Expression;
pub use expression::FunctionCallArguments;
pub use expression::ParameterList;
pub use expression::TableField;
pub use expression::TableFieldKeyValue;
pub use expression::TableFieldNameValue;
pub use expression::TableFieldValue;

pub use statement::AttName;
pub use statement::Attrib;
pub use statement::Block;
pub use statement::FunctionName;
pub use statement::ReturnStatement;
pub use statement::Statement;
pub use statement::StmtAssignment;
pub use statement::StmtBreak;
pub use statement::StmtDo;
pub use statement::StmtElseIf;
pub use statement::StmtFor;
pub use statement::StmtForGeneric;
pub use statement::StmtFunctionCall;
pub use statement::StmtFunctionDefinition;
pub use statement::StmtFunctionDefinitionLocal;
pub use statement::StmtGoto;
pub use statement::StmtIf;
pub use statement::StmtLabel;
pub use statement::StmtLocalDeclaration;
pub use statement::StmtNone;
pub use statement::StmtRepeat;
pub use statement::StmtWhile;

pub use lua_tokenizer::Tokenizer;

pub use parser::ChunkOrExpressionsContext as Context;
pub use parser::ChunkOrExpressionsParser as Parser;

/// for interpreter to handle both chunk and expression
#[derive(Debug, Clone)]
pub enum ChunkOrExpressions {
    Chunk(Block),
    Expressions(Vec<Expression>),
}

/// parse lua source code to AST
pub fn parse_str(source: &str) -> Result<Block, ParseError> {
    parse_bytes(source.as_bytes())
}

/// parse lua source code to AST
pub fn parse_bytes(source: &[u8]) -> Result<Block, ParseError> {
    let tokenizer = Tokenizer::from_bytes(source);
    let mut context = parser::ChunkOrExpressionsContext::new(());

    for token in tokenizer.into_iter() {
        let token = match token {
            Ok(token) => token,
            Err(e) => {
                return Err(ParseError::TokenizeError(e));
            }
        };

        match context.feed(token) {
            Ok(_) => {}
            Err(err) => {
                let (expected_terms, expected_nonterms) = context.expected_token_str();
                let error = InvalidToken {
                    token: Some(err.term.into_term().unwrap()),
                    expected: expected_terms.collect(),
                    expected_nonterm: expected_nonterms.collect(),
                };
                return Err(ParseError::InvalidToken(error));
                // parser::ChunkOrExpressionsParseError::ReduceAction(reduce_actions) => {
                //     return Err(reduce_actions.into_iter().next().unwrap());
                // }
            }
        }
    }

    let mut block = None;

    let (expected_terms, expected_nonterms) = context.expected_token_str();
    let res = context.accept_all();
    match res {
        Ok(matched) => {
            for (matched, _) in matched {
                match matched {
                    ChunkOrExpressions::Chunk(block_) => {
                        if block.is_some() {
                            return Err(ParseError::Ambiguous);
                        }
                        block = Some(block_);
                    }
                    _ => {}
                }
            }
        }
        Err(_) => {
            let error = InvalidToken {
                token: None,
                expected: expected_terms.collect(),
                expected_nonterm: expected_nonterms.collect(),
            };
            return Err(ParseError::InvalidToken(error));
        }
    }

    if let Some(block) = block {
        Ok(block)
    } else {
        Err(ParseError::Ambiguous)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_table_constructor_with_trailing_field_separator() {
        let source = r#"
            local a = {}
            local b = { 1, 2, }
            local c = { name = "lua"; [1] = true; }
        "#;

        assert!(crate::parse_str(source).is_ok());
    }
}
