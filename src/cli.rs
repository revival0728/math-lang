use crate::env::*;
use crate::runtime::Runtime;
use clap::Parser;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

// TODO: improve and fix error message

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct ExArgs {
    /// Path of source file
    source: Option<PathBuf>,

    /// ENV PRECISION
    #[arg(long, value_name = "VALUE")]
    env_precision: Option<u32>,

    /// ENV DETAIL_DEPTH
    #[arg(long, value_name = "VALUE")]
    env_detail_depth: Option<u32>,

    /// ENV MAX_STACK_DEPTH
    #[arg(long, value_name = "VALUE")]
    env_max_stack_depth: Option<u32>,

    /// Maximum stack size
    #[arg(long, value_name = "BYTES")]
    pub max_stack_size: Option<usize>,
}

pub struct CLI<'cli> {
    info: String,
    line_prefix: String,
    input: Vec<Box<str>>,
    runtime: Runtime<'cli>,
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

        Self {
            info,
            line_prefix,
            input,
            runtime,
        }
    }
    fn exec_source(&mut self, source: String, repl: bool) {
        let source = Box::leak(source.into_boxed_str());
        // SAFE: The Box<> will not free before Runtime<>
        unsafe { self.input.push(Box::from_raw(source)) };
        match self.runtime.execute(source) {
            Ok(output) => {
                if repl {
                    if let Some(out) = output.last() {
                        println!("{}", out);
                    }
                } else {
                    for out in output {
                        println!("{}", out);
                    }
                }
            }
            Err(err) => {
                println!(
                    "{}",
                    if repl {
                        err.no_loc_info()
                    } else {
                        err.all_info()
                    }
                );
            }
        };
    }
    fn set_env(&mut self, args: &ExArgs) {
        unsafe {
            PRECISION = args.env_precision.unwrap_or(PRECISION);
            DETAIL_DEPTH = args.env_detail_depth.unwrap_or(DETAIL_DEPTH);
            MAX_STACK_DEPTH = args.env_max_stack_depth.unwrap_or(MAX_STACK_DEPTH);
        }
    }
    pub fn run(&mut self, args: ExArgs) {
        if let Some(source) = args.source.clone() {
            self.set_env(&args);
            let source = match std::fs::read_to_string(source) {
                Ok(s) => s,
                Err(e) => {
                    println!("File Error: {}", e);
                    return;
                }
            };
            self.exec_source(source, false);
            return;
        }

        println!("{}", self.info);
        self.set_env(&args);
        unsafe { DETAIL_DEPTH = 1 };
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
            if input.trim().is_empty() {
                continue;
            }
            if input.trim() == "quit" || input.trim() == "exit" {
                break;
            }
            self.exec_source(input, true);
        }
    }
}
