use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use colored::Colorize;
use driver::{Driver, Parser};
use tempfile::tempdir;

fn main() {
    let mut argv = std::env::args().skip(1);
    let filter = argv.next().filter(|f| !f.starts_with("--"));

    // If the first arg was a flag, it belongs to the driver, not the filter.
    let driver_flags: Vec<String> = std::env::args()
        .skip(1 + usize::from(filter.is_some()))
        .collect();

    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");

    let mut entries: Vec<PathBuf> = std::fs::read_dir(&fixtures_dir)
        .expect("Failed to read tests/fixtures folder")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("p7"))
        .collect();
    entries.sort();

    if let Some(pattern) = &filter {
        entries.retain(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .contains(pattern.as_str())
        });
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

    let (mut passed, mut failed) = (0, 0);
    let single = entries.len() == 1;

    for path in &entries {
        let name = path.file_name().unwrap().to_string_lossy();
        match run_test(path, &driver_flags, single) {
            Ok(Some(code)) => {
                passed += 1;
                println!(
                    "  {} {} {}",
                    "✓ PASS".green().bold(),
                    name,
                    format!("(exit: {code})").dimmed()
                );
            }
            Ok(None) => {
                passed += 1;
                println!(
                    "  {} {} {}",
                    "✓ PASS".green().bold(),
                    name,
                    "(stage ok)".dimmed()
                );
            }
            Err(err) => {
                failed += 1;
                println!(
                    "  {} {}\n        {}",
                    "✗ FAIL".red().bold(),
                    name,
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

fn run_test(file_path: &Path, extra_flags: &[String], single: bool) -> Result<Option<i32>, String> {
    let source =
        std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {e}"))?;

    let expected_exit = source
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("// EXIT:")?
                .trim()
                .parse::<i32>()
                .ok()
        })
        .ok_or("Missing '// EXIT: <code>' header at top of file")?;

    let temp_dir = tempdir().map_err(|e| format!("Tempdir creation failed: {e}"))?;
    let bin_path = temp_dir.path().join("test_bin");

    let mut argv = vec![
        "p7c".to_string(),
        file_path.display().to_string(),
        "-o".to_string(),
        bin_path.display().to_string(),
    ];
    argv.extend_from_slice(extra_flags);

    let args = Driver::try_parse_from(&argv).map_err(|e| format!("Bad driver options: {e}"))?;

    driver::run(&args, &mut std::io::sink()).map_err(|e| format!("{e:#}"))?;

    let mut out: Box<dyn Write> = if single && args.stops_early() {
        Box::new(std::io::stdout())
    } else {
        Box::new(std::io::sink())
    };

    driver::run(&args, &mut out).map_err(|e| format!("{e:#}"))?;
    out.flush().ok();

    if args.stops_early() {
        return Ok(None);
    }

    let output = Command::new(&bin_path)
        .output()
        .map_err(|e| format!("Failed to run binary: {e}"))?;
    let actual_exit = output.status.code().unwrap_or(-1);

    if actual_exit == expected_exit {
        Ok(Some(actual_exit))
    } else {
        Err(format!(
            "Exit code mismatch: expected {expected_exit}, got {actual_exit}"
        ))
    }
}
