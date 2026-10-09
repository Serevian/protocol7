use protocol7_compiler::Driver;
use std::process::Command;

fn main() {
    let source = include_str!("../../tests/ui/basic.p7");

    let asm = Driver::new().compile(source).unwrap();

    std::fs::write("output.s", &asm);

    println!("Generated assembly saved to output.s");

    // 6. Shell out to GCC / Clang to assemble and link
    let status = Command::new("gcc")
        .arg("output.s")
        .arg("-o")
        .arg("output")
        .status()
        .unwrap();

    if !status.success() {
        eprintln!("Linking failed!");
        std::process::exit(1);
    }

    println!("Successfully compiled executable: basic.p7");
}
