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

    /// ENV PRINT_SET_INST
    #[arg(long, value_name = "VALUE")]
    env_print_set_inst: Option<u32>,

    /// ENV DETAIL_DEPTH
    #[arg(long, value_name = "VALUE")]
    env_detail_depth: Option<u32>,

    /// ENV MAX_STACK_DEPTH
    #[arg(long, value_name = "VALUE")]
    env_max_stack_depth: Option<u32>,

    /// ENV INDEX_BASE
    #[arg(long, value_name = "VALUE")]
    env_index_base: Option<u32>,

    /// Maximum stack size
    #[arg(long, value_name = "BYTES")]
    pub max_stack_size: Option<usize>,
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
    fn new_source(&mut self, source: String) -> &'static str {
        let source = Box::leak(source.into_boxed_str());
        // SAFE: The Box<> will not free before Runtime<>
        unsafe { self.input.push(Box::from_raw(source)) };
        source
    }
    fn set_env(&mut self, args: &ExArgs) {
        macro_rules! set {
            ($uenv:ident, $lenv:ident, $val:expr) => {
                unsafe {
                    let source = self.new_source(format!(
                        "__{}__({})",
                        stringify!($lenv),
                        $val.unwrap_or($uenv)
                    ));
                    match self.runtime.execute(source) {
                        Ok(_) => self.cur_output += 1,
                        Err(e) => println!("Exection Parameter Error:\n\t{}", e.no_loc_info()),
                    };
                }
            };
        }
        set!(PRECISION, precision, args.env_precision);
        set!(PRINT_SET_INST, print_set_inst, args.env_print_set_inst);
        set!(DETAIL_DEPTH, detail_depth, args.env_detail_depth);
        set!(MAX_STACK_DEPTH, max_stack_depth, args.env_max_stack_depth);
        set!(INDEX_BASE, index_base, args.env_index_base);
    }
    fn exec_source(&mut self, source: String, repl: bool) {
        let source = self.new_source(source);
        match self.runtime.execute(source) {
            Ok(output) => {
                if repl {
                    if let Some(out) = output.get(self.cur_output) {
                        println!("{}", out);
                        self.cur_output += 1;
                    }
                } else {
                    for out in output[self.cur_output..].iter() {
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
        if let Some(source) = args.source.clone() {
            let work_path = match source.parent() {
                Some(path) => path.to_path_buf(),
                None => PathBuf::new(),
            };
            self.runtime.set_work_path(work_path);
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
