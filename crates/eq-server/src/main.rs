use clap::Parser;

#[derive(Parser)]
#[command(name = "ritma", version)]
struct Cli {}

fn main() {
    Cli::parse();
}
