use protocol7_compiler::Driver;

fn main() {
    let source = include_str!("../../tests/ui/basic.p7");

    let driver = Driver::new().compile(source);
}
