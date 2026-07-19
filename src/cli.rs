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

    /// ENV PRINT_SET_ENV
    #[arg(long, value_name = "VALUE")]
    env_print_set_env: Option<u32>,

    /// ENV DETAIL_DEPTH
    #[arg(long, value_name = "VALUE")]
    env_detail_depth: Option<u32>,

    /// ENV MAX_STACK_DEPTH
    #[arg(long, value_name = "VALUE")]
    env_max_stack_depth: Option<u32>,

    /// Maximum stack size
    #[arg(long, value_name = "BYTES")]
    pub max_stack_size: Option<usize>,

    /// Flatten instructions into list
    #[arg(long, value_name = "VALUE")]
    flatten_inst: Option<u32>,
}

pub struct CLI<'cli> {
    info: String,
    line_prefix: String,
    input: Vec<Box<str>>,
    runtime: Runtime<'cli>,
    cur_output: usize,
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
        let cur_output = 0;

        Self {
            info,
            line_prefix,
            input,
            runtime,
            cur_output,
        }
    }
    fn set_env(&mut self, args: &ExArgs) {
        unsafe {
            PRECISION = args.env_precision.unwrap_or(PRECISION);
            PRINT_SET_INST = args.env_print_set_env.unwrap_or(PRINT_SET_INST);
            DETAIL_DEPTH = args.env_detail_depth.unwrap_or(DETAIL_DEPTH);
            MAX_STACK_DEPTH = args.env_max_stack_depth.unwrap_or(MAX_STACK_DEPTH);
            FLATTEN_INST = args.flatten_inst.unwrap_or(FLATTEN_INST);
        }
    }
    fn exec_source(&mut self, source: String, repl: bool) {
        let source = Box::leak(source.into_boxed_str());
        // SAFE: The Box<> will not free before Runtime<>
        unsafe { self.input.push(Box::from_raw(source)) };
        match self.runtime.execute(source) {
            Ok(output) => {
                if repl {
                    if let Some(out) = output.get(self.cur_output) {
                        println!("{}", out);
                        self.cur_output += 1;
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
    pub fn run(&mut self, args: ExArgs) {
        // let mut args = args;
        // args.source = Some(
        //     std::path::PathBuf::new()
        //         .join(".")
        //         .join("scripts")
        //         .join("test_stack.mls"),
        // );
        if let Some(source) = args.source.clone() {
            unsafe { PRINT_SET_INST = 0 };
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
