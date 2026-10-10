use anyhow::{Context, Result, bail};
use clap::Parser;
use colored::Colorize;
use p7c::*;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Parser, Debug)]
#[command(name = "p7c", author, version, about = "P7 Compiler")]
struct Driver {
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long)]
    lex: bool,

    #[arg(long)]
    parse: bool,

    #[arg(long)]
    codegen: bool,

    #[arg(short = 'S', long)]
    emit_asm: bool,
}

fn main() -> Result<()> {
    let args = Driver::parse();

    let source = std::fs::read_to_string(&args.input)
        .with_context(|| format!("Could not read input file: {}", args.input.display()))?;

    let tokens = lexer::Lexer::new(&source).tokenize();
    if args.lex {
        println!("{:#?}", tokens);
        return Ok(());
    }

    let mut parser = parser::Parser::new(tokens);
    let ast = parser
        .parse()
        .map_err(|err| anyhow::anyhow!("Parse Error: {err}"))?;
    if args.parse {
        println!("{:#?}", ast);
        return Ok(());
    }

    let mut generator = assembly_generator::AssemblyGenerator::new(ast);
    let asm_ast = generator.generate();
    if args.codegen {
        println!("{:#?}", asm_ast);
        return Ok(());
    }

    let asm_text = code_emitter::Emitter::new().emit_program(&asm_ast);

    let binary_name = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("");
        path
    });

    let asm_path = binary_name.with_extension("s");

    std::fs::write(&asm_path, &asm_text)
        .with_context(|| format!("Failed to write assembly to {}", asm_path.display()))?;

    if args.emit_asm {
        println!("{} {}", "Created assembly:".green(), asm_path.display());
        return Ok(());
    }

    assemble_and_link(&asm_path, &binary_name)?;

    let _ = std::fs::remove_file(&asm_path);

    println!(
        "{} {}",
        "Successfully compiled:".green().bold(),
        binary_name.display()
    );
    Ok(())
}

fn assemble_and_link(asm_file: &Path, output_file: &Path) -> Result<()> {
    let compiler = if Command::new("gcc").arg("--version").output().is_ok() {
        "gcc"
    } else {
        "clang"
    };

    let status = Command::new(compiler)
        .arg(asm_file)
        .arg("-o")
        .arg(output_file)
        .status()
        .with_context(|| format!("Failed to invoke {compiler}"))?;

    if !status.success() {
        bail!(
            "Assembler/Linker ({compiler}) failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(())
}
