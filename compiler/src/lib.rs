mod assembly_ast;
mod assembly_generator;
mod ast;
mod code_emitter;
mod lexer;
mod parser;
mod span;
mod token;

pub struct Driver {}

impl Driver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        let tokens = lexer::Lexer::new(source).tokenize();

        let mut parser = parser::Parser::new(tokens);
        let ast = parser.parse().map_err(|e| format!("Parser error: {e}"))?;

        let mut generator = assembly_generator::AssemblyGenerator::new(ast);
        let asm_ast = generator.generate();

        // 4. Assembly AST -> String
        let asm = code_emitter::Emitter::new().emit_program(&asm_ast);

        Ok(asm)
    }
}
