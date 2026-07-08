use crate::env;
use crate::runtime::Runtime;
use clap::Parser;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

// TODO: improve and fix error message

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct ExArgs {
    /// Path of source file
    source: Option<PathBuf>,

    /// ENV PRECISION
    #[arg(long, value_name = "VALUE")]
    env_precision: Option<u32>,

    /// ENV DETAIL_DEPTH
    #[arg(long, value_name = "VALUE")]
    env_detail_depth: Option<u32>,
}

pub struct CLI<'cli> {
    info: String,
    line_prefix: String,
    input: Vec<Box<str>>,
    runtime: Runtime<'cli>,
    args: ExArgs,
}

impl<'cli> CLI<'cli> {
    pub fn new() -> Self {
        let info = format!(
            "Math-Lang {} [{} rustc-{}] on {}",
            env!("CARGO_PKG_VERSION"),
            env!("VERGEN_BUILD_DATE"),
            env!("VERGEN_RUSTC_SEMVER"),
            env!("VERGEN_CARGO_TARGET_TRIPLE"),
        );
        let line_prefix = format!(">> ");
        let input = Vec::new();
        let runtime = Runtime::new();
        let args = ExArgs::parse();

        Self {
            info,
            line_prefix,
            input,
            runtime,
            args,
        }
    }
    fn exec_source(&mut self, source: String, loc_info: bool) {
        let source = Box::leak(source.into_boxed_str());
        // SAFE: The Box<> will not free before Runtime<>
        unsafe { self.input.push(Box::from_raw(source)) };
        match self.runtime.execute(source) {
            Ok(output) => {
                if let Some(out) = output.last() {
                    println!("{}", out);
                }
            }
            Err(err) => {
                println!(
                    "{}",
                    if loc_info {
                        err.all_info()
                    } else {
                        err.no_loc_info()
                    }
                );
            }
        };
    }
    pub fn run(&mut self) {
        if let Some(source) = self.args.source.clone() {
            let source = match std::fs::read_to_string(source) {
                Ok(s) => s,
                Err(e) => {
                    println!("File Error: {}", e);
                    return;
                }
            };
            self.exec_source(source, true);
            return;
        }

        println!("{}", self.info);
        unsafe { env::DETAIL_DEPTH = 1 };
        loop {
            print!("{}", self.line_prefix);
            match io::stdout().flush() {
                Ok(_) => {}
                Err(e) => println!("IO Error: {}", e),
            };
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {}
                Err(e) => println!("IO Error: {}", e),
            };
            if input.trim() == "quit" || input.trim() == "exit" {
                break;
            }
            self.exec_source(input, false);
        }
    }
}
