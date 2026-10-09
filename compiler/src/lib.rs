mod ast;
mod lexer;
mod parser;
mod span;
mod token;

pub struct Driver {}

impl Driver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, source: &str) {
        let tokens = lexer::Lexer::new(source).tokenize();

        let ast = parser::Parser::new(tokens).parse().unwrap();

        println!("{ast:?}");
    }
}
