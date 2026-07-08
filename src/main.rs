mod cli;
mod comiler;
mod env;
mod error;
mod lexer;
mod runtime;
mod test;
mod var;

fn main() {
    let mut cli = cli::CLI::new();
    cli.run();
}
