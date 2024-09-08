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

// re-exports
pub use lua_tokenizer::FloatType;
pub use lua_tokenizer::IntOrFloat;
pub use lua_tokenizer::IntType;
pub use lua_tokenizer::Span;
pub use lua_tokenizer::Token;

pub use error::InvalidToken;
pub use error::ParseError;
pub use error::TokenizeError;

pub use expression::ExprBinary;
pub use expression::ExprBool;
pub use expression::ExprFunction;
pub use expression::ExprFunctionCall;
pub use expression::ExprIdent;
pub use expression::ExprNil;
pub use expression::ExprNumeric;
pub use expression::ExprString;
pub use expression::ExprTable;
pub use expression::ExprTableIndex;
pub use expression::Expression;
pub use expression::ParameterList;
pub use expression::TableField;

pub use statement::AttName;
pub use statement::Attrib;
pub use statement::Block;
pub use statement::FunctionName;
pub use statement::ReturnStatement;
pub use statement::Statement;
pub use statement::StmtAssignment;
pub use statement::StmtBreak;
pub use statement::StmtDo;
pub use statement::StmtFor;
pub use statement::StmtForGeneric;
pub use statement::StmtFunctionCall;
pub use statement::StmtFunctionDefinition;
pub use statement::StmtFunctionDefinitionLocal;
pub use statement::StmtGoto;
pub use statement::StmtIf;
pub use statement::StmtLabel;
pub use statement::StmtLocalDeclaration;
pub use statement::StmtRepeat;
pub use statement::StmtWhile;

pub use lua_tokenizer::Tokenizer;

/// parse lua source code to AST
pub fn parse_str(source: &str) -> Result<Block, ParseError> {
    let tokenizer = Tokenizer::new(source);
    let parser = parser::ChunkParser::new();
    let mut context = parser::ChunkContext::new();

    for token in tokenizer {
        let token = match token {
            Ok(token) => token,
            Err(e) => {
                return Err(ParseError::TokenizeError(e));
            }
        };

        match context.feed(&parser, token, &mut ()) {
            Ok(_) => {}
            Err(err) => {
                let token = err.term;
                let expected = context.expected(&parser).cloned().collect::<Vec<_>>();
                let expected_nonterm = context
                    .expected_nonterm(&parser)
                    .cloned()
                    .collect::<Vec<_>>();
                let error = InvalidToken {
                    token: Some(token),
                    expected,
                    expected_nonterm,
                };
                return Err(ParseError::InvalidToken(error));
            }
        }
    }
    // feed eof
    let eof_token = lua_tokenizer::Token {
        token_type: lua_tokenizer::TokenType::Eof,
        span: Span::new(source.len(), source.len()),
    };
    match context.feed(&parser, eof_token, &mut ()) {
        Ok(_) => {}
        Err(_) => {
            let expected = context.expected(&parser).cloned().collect::<Vec<_>>();
            let expected_nonterm = context
                .expected_nonterm(&parser)
                .cloned()
                .collect::<Vec<_>>();
            let error = InvalidToken {
                token: None,
                expected,
                expected_nonterm,
            };
            return Err(ParseError::InvalidToken(error));
        }
    }

    match context.accept() {
        Ok(block) => Ok(block),
        Err(_) => {
            return Err(ParseError::Ambiguous);
        }
    }
}
