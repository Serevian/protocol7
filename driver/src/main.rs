use driver::{Driver, Parser};

fn main() -> anyhow::Result<()> {
    let args = Driver::parse();
    driver::run(&args, &mut std::io::stdout())
}
