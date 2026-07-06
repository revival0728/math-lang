mod cli;
mod comiler;
mod error;
mod lexer;
mod runtime;
mod test;

fn main() {
    let mut cli = cli::CLI::new();
    cli.run();
}
