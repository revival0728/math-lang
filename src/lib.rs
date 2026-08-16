//! # Math Lang Library Document
//! For now, only [`Rust Module API`](rmapi) is documented.
//!
//! ## Writing Rust Library Module
//! Please checkout [`Rust Module API`](rmapi) document.
//!
#![doc = include_str!("../docs/README.md")]
mod big_uint;
pub mod builtin;
pub mod comiler;
pub mod env;
mod error;
mod lexer;
pub mod module;
pub mod rmapi;
pub mod runtime;
mod test;
mod var;

pub use rmapi::*;
