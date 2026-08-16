mod cli;
use clap::Parser;
use std::thread;

fn main() {
    let args = cli::ExArgs::parse();

    let builder = thread::Builder::new()
        .name("runtime".into())
        .stack_size(args.max_stack_size.unwrap_or(64 * 1024 * 1024));

    let cli = builder
        .spawn(move || {
            cli::CLI::new().run(args);
        })
        .unwrap();

    cli.join().unwrap();
}
