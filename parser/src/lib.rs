mod expression;
mod statement;
// mod parser;
mod parser_expanded;
use parser_expanded as parser;
mod types;

pub use expression::Expression;
pub use statement::Block;
pub use statement::ReturnStatement;
pub use statement::Statement;

pub use lua_tokenizer::tokenize;

pub fn parse_str(source: &str) -> Result<Block, ()> {
    let mut tokens = match lua_tokenizer::tokenize(&source) {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("tokenize error: {}", err);
            return Err(());
        }
    };
    tokens.push(lua_tokenizer::Token {
        token_type: lua_tokenizer::TokenType::Eof,
        byte_start: source.len(),
        byte_end: source.len(),
    });

    let parser = parser::ChunkParser::new();
    let mut context = parser::ChunkContext::new();

    for token in tokens.into_iter() {
        match context.feed(&parser, token, &mut ()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("parse error: {}", err);
                return Err(());
            }
        }
    }

    match context.accept() {
        Ok(block) => Ok(block),
        Err(err) => {
            eprintln!("accept error: {}", err);
            return Err(());
        }
    }
}
