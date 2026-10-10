use std::{
    path::{Path, PathBuf},
    process::Command,
};

use colored::Colorize;
use tempfile::tempdir;

fn main() {
    // Optional filter for which tests to run
    let filter = std::env::args().nth(1);

    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&fixtures_dir)
        .expect("Failed to read tests/fixtures folder")
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("p7"))
        .collect();

    entries.sort();

    // Apply filter if it exists
    if let Some(ref pattern) = filter {
        entries.retain(|p| p.file_name().unwrap().to_string_lossy().contains(pattern));
    }

    if entries.is_empty() {
        println!("{}", "No matching .p7 test fixtures found!".yellow());
        return;
    }

    println!(
        "\n{} {} test fixtures...\n",
        "Running".bold().cyan(),
        entries.len()
    );

    let mut passed = 0;
    let mut failed = 0;

    for path in &entries {
        match run_test(path) {
            Ok(code) => {
                passed += 1;
                println!(
                    "  {} {} {} {}",
                    "✓ PASS".green().bold(),
                    path.file_name().unwrap().to_string_lossy(),
                    "(exit:".dimmed(),
                    format!("{code})").dimmed()
                );
            }
            Err(err) => {
                failed += 1;
                println!(
                    "  {} {}\n        {}",
                    "✗ FAIL".red().bold(),
                    path.file_name().unwrap().to_string_lossy(),
                    err.replace('\n', "\n        ")
                );
            }
        }
    }

    println!("\n{}", "─".repeat(50).dimmed());
    if failed == 0 {
        println!("{}", format!("All {passed} tests passed!").green().bold());
    } else {
        println!(
            "{}",
            format!("{passed} passed, {failed} failed").red().bold()
        );
        std::process::exit(1);
    }
}

fn run_test(file_path: &Path) -> Result<i32, String> {
    let source =
        std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {e}"))?;

    let expected_exit = source
        .lines()
        .find_map(|line| {
            let t = line.trim();
            if t.starts_with("// EXIT:") {
                t.trim_start_matches("// EXIT:").trim().parse::<i32>().ok()
            } else {
                None
            }
        })
        .ok_or_else(|| "Missing '// EXIT: <code>' header at top of file".to_string())?;

    // 2. In-memory compilation pipeline
    let tokens = p7c::lexer::Lexer::new(&source).tokenize();

    let mut parser = p7c::parser::Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parser Error: {e}"))?;

    let mut generator = p7c::aast_generator::AssemblyGenerator::new(ast);
    let asm_ast = generator.generate();

    let asm_text = p7c::emitter::Emitter::new().emit_program(&asm_ast);

    // 3. Assemble and Link in a temporary directory
    let temp_dir = tempdir().map_err(|e| format!("Tempdir creation failed: {e}"))?;
    let asm_path = temp_dir.path().join("out.s");
    let bin_path = temp_dir.path().join("test_bin");

    std::fs::write(&asm_path, &asm_text).map_err(|e| format!("Failed to write .s: {e}"))?;

    let gcc_status = Command::new("gcc")
        .arg(&asm_path)
        .arg("-o")
        .arg(&bin_path)
        .status()
        .map_err(|e| format!("Failed to invoke GCC: {e}"))?;

    if !gcc_status.success() {
        return Err("GCC assembly/linking failed".to_string());
    }

    let output = Command::new(&bin_path)
        .output()
        .map_err(|e| format!("Failed to run binary: {e}"))?;

    let actual_exit = output.status.code().unwrap_or(-1);

    if actual_exit == expected_exit {
        Ok(actual_exit)
    } else {
        Err(format!(
            "Exit code mismatch: expected {expected_exit}, got {actual_exit}"
        ))
    }
}
