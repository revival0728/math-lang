mod comiler;
mod env;
mod error;
mod lexer;
mod runtime;
mod test;
mod var;

use runtime::Runtime;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn execute(source: String) -> Result<Vec<String>, String> {
    let mut runtime = Runtime::new();
    match runtime.execute(&source) {
        Ok(out) => Ok(out.clone()),
        Err(err) => Err(err.all_info()),
    }
}
