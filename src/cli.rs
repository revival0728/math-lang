use crate::runtime::Runtime;
use std::io;
use std::io::prelude::*;

// TODO: improve and fix error message

pub struct CLI<'cli> {
    info: String,
    line_prefix: String,
    input: Vec<Box<str>>,
    runtime: Runtime<'cli>,
}

impl<'cli> CLI<'cli> {
    pub fn new() -> Self {
        let info = format!(
            "Math-Lang {} [{} rustc-{} on {}]",
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
    pub fn run(&mut self) {
        println!("{}", self.info);
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
            let source = Box::leak(input.into_boxed_str());
            unsafe {
                self.input.push(Box::from_raw(source));
            }
            match self.runtime.execute(source) {
                Ok(output) => {
                    if let Some(out) = output.last() {
                        println!("{}", out);
                    }
                }
                Err(err) => {
                    println!("{}", err.all_info());
                }
            };
        }
    }
}
