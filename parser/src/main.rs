use lua_tokenizer::{Token, TokenType};

mod expression;
mod statement;
// mod parser;
mod parser_expanded;
use parser_expanded as parser;
mod types;

pub use expression::Expression;
pub use statement::Statement;

fn main() {
    let filename = std::env::args().nth(1).expect("no filename given");
    let source = std::fs::read_to_string(&filename).expect("failed to read file");

    let mut tokens = match lua_tokenizer::tokenize(&source) {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    };
    tokens.push(Token {
        token_type: TokenType::Eof,
        byte_start: 0,
        byte_end: 0,
    });
    for token in &tokens {
        println!("{}", token);
    }

    let parser = parser::ChunkParser::new();
    let mut context = parser::ChunkContext::new();

    for token in tokens.into_iter() {
        match context.feed(&parser, token, &mut ()) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("error: {}", context);
                eprintln!("error: {}", err);
                std::process::exit(1);
            }
        }
    }

    let res = context.accept_all().collect::<Vec<_>>();
    for block in res.into_iter() {
        println!("{:?}", block);
    }
}
