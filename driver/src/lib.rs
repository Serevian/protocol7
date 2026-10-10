use anyhow::{Context, Result, bail};
pub use clap::Parser;
use colored::Colorize;
use p7c::*;
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Parser, Debug)]
#[command(name = "p7c", author, version, about = "P7 Compiler")]
pub struct Driver {
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long)]
    lex_table: bool,

    #[arg(long)]
    lex_lines: bool,

    #[arg(long)]
    parse: bool,

    #[arg(long)]
    ptac: bool,

    #[arg(long)]
    codegen: bool,

    #[arg(short = 'S', long)]
    emit_asm: bool,
}

impl Driver {
    pub fn stops_early(&self) -> bool {
        self.lex_table || self.lex_lines || self.parse || self.ptac || self.codegen || self.emit_asm
    }
}

pub fn run(args: &Driver, out: &mut dyn Write) -> Result<()> {
    let source = std::fs::read_to_string(&args.input)
        .with_context(|| format!("Could not read input file: {}", args.input.display()))?;

    let tokens = lexer::Lexer::new(&source).tokenize();
    if args.lex_table {
        let string = token::format_token_table(&tokens);
        writeln!(out, "{string}")?;
        return Ok(());
    }
    if args.lex_lines {
        let string = token::format_token_lines(&tokens);
        writeln!(out, "{string}")?;
        return Ok(());
    }

    let mut parser = parser::Parser::new(tokens);
    let ast = parser
        .parse()
        .map_err(|err| anyhow::anyhow!("Parse Error: {err}"))?;
    if args.parse {
        writeln!(out, "{}", ast.dump_tree())?;
        return Ok(());
    }

    let ptac_generator = ptac_generator::PtacGenerator::new(ast);
    let ptac = ptac_generator.generate();
    if args.ptac {
        writeln!(out, "{}", ptac)?;
        return Ok(());
    }

    let mut assembly_generator = aast_generator::AssemblyGenerator::new(ptac);
    let asm_ast = assembly_generator.generate();
    if args.codegen {
        writeln!(out, "{}", asm_ast)?;
        return Ok(());
    }

    let asm_text = emitter::Emitter::new().emit_program(&asm_ast);

    let binary_name = args.output.clone().unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("");
        path
    });

    let asm_path = binary_name.with_extension("s");
    if args.emit_asm {
        writeln!(
            out,
            "{} {}",
            "Created assembly:".green(),
            asm_path.display()
        )?;
        writeln!(out, "\n{}", pretty_asm(&asm_text))?;
        return Ok(());
    }

    std::fs::write(&asm_path, &asm_text)
        .with_context(|| format!("Failed to write assembly to {}", asm_path.display()))?;

    if args.emit_asm {
        writeln!(
            out,
            "{} {}",
            "Created assembly:".green(),
            asm_path.display()
        )?;
        return Ok(());
    }

    assemble_and_link(&asm_path, &binary_name)?;

    let _ = std::fs::remove_file(&asm_path);

    writeln!(
        out,
        "{} {}",
        "Successfully compiled:".green().bold(),
        binary_name.display()
    )?;
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

fn pretty_asm(asm: &str) -> String {
    asm.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let indent = &line[..line.len() - trimmed.len()];

            if trimmed.is_empty() {
                String::new()
            } else if trimmed.starts_with('#') || trimmed.starts_with("//") {
                format!("{indent}{}", trimmed.dimmed())
            } else if trimmed.ends_with(':') {
                // Labels, including local ones like `.L0:`
                format!("{indent}{}", trimmed.yellow().bold())
            } else if trimmed.starts_with('.') {
                // Directives: .globl, .text, .section ...
                format!("{indent}{}", trimmed.magenta())
            } else {
                // Instruction: colour the mnemonic, leave operands plain.
                match trimmed.split_once(char::is_whitespace) {
                    Some((mnemonic, operands)) => {
                        format!(
                            "{indent}{} {}",
                            mnemonic.cyan().bold(),
                            operands.trim_start()
                        )
                    }
                    None => format!("{indent}{}", trimmed.cyan().bold()),
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
