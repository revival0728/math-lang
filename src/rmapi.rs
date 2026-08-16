#![allow(unused)]

//! # Rust Module API
//! This module provides Rust APIs for math-lang rust library module
//!
//! ## Writing Rust Functions for math-lang Runtime
//! The function must be type [`RMFunPtr`] which is `fn(ScopeApi) -> Result<Option<VarApi>, String>`
//! where `String` contains error message returning to math-lang runtime.
//!
//! The current scope of [`ScopeApi`] contains arguments of math-lang the function
//! which defined in [`crate::export!`] or [`ModMember`].
//!
//! For example:
//! ```
//! pub fn plus_one(sapi: ScopeApi) -> RMFunRetType {
//!     let x: i32 = sapi
//!         .get_current_var("x")
//!         .unwrap()  // always valid because it is the function parameter
//!         .try_into()
//!         .map_err(|_| format!("ONLY I32 CAN BE PLUS ONE"))?;
//!     Ok(Some(VarApi::from(x + 1)))
//! }
//!
//! export! { plus_one(x) = plus_one; }
//! ```
//!
//! ## Rust Module Define Example
//!
//! `src/lib.rs`:
//! ```
//! use math_lang::prelude::*;
//!
//! pub fn rust_add(left: i64, right: i64) -> i64 {
//!     left + right
//! }
//!
//! pub fn add(sapi: ScopeApi) -> RMFunRetType {
//!     let a: i64 = sapi
//!         .get_current_var("a")
//!         .unwrap()
//!         .try_into()
//!         .map_err(|t| format!("expected I64 type got {} type", t))?;
//!     let b: i64 = sapi
//!         .get_current_var("b")
//!         .unwrap()
//!         .try_into()
//!         .map_err(|t| format!("expected I64 type got {} type", t))?;
//!     Ok(Some(VarApi::from(rust_add(a, b))))
//! }
//!
//! export! {
//!     LUCKY = I32(0923);
//!     add_i64(a, b) = add;
//! }
//! ```
//!
//! `Cargo.toml`:
//! ```
//! [package]
//! name = "rlib"
//! version = "0.1.0"
//! edition = "2024"
//!
//! [dependencies]
//! math-lang = { path = "path to math-lang crate" }
//!
//! [lib]
//! crate-type = ["dylib"]
//! ```
//!
//! For more examples please checks out the source of [`crate::builtin`]
//!
//! ## NOTICE
//! - The file extension of dynamic library could be arbitary, but highly recommends that `windows` uses `.dll`, `linux` uses `.so` and `macos` uses `.dylib`
//! - The rust library module should be compiled with the same Rust compiler version as math-lang binary
use crate::runtime::{Fun, Scope};
use crate::var::{Var, VarType};
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

/// Quick import for Rust Module API
pub mod prelude {
    pub use super::{ModMember, Number, RMExport, RMFunRetType, ScopeApi, VarApi};
    pub use crate::export;
}

/// Result type of rust library module functions
pub type RMFunResult<T> = Result<T, String>;
/// Return type of rust library module functions
pub type RMFunRetType = RMFunResult<Option<VarApi>>;
/// Function pointer type of rust library module
pub type RMFunPtr = fn(ScopeApi) -> RMFunRetType;
/// Export type of rust library module
pub type RMExport = Vec<ModMember>;
/// Export function type of rust library module
pub type RMExportFun = fn() -> RMExport;

#[macro_export]
/// Export rust library module members
///
/// This macro implements a tiny script language to help you export the module.
///
/// The syntax is simple:
/// - export variable: `$export_name = $type($value);` where `$type` is the variants of [`Number`]
/// - export function: `$export_name($para_names) = $rust_name;`
/// - don't forget the semicolons!
///
/// The Example:
/// ```
/// export! {
///     pi = F64(3.14);
///     sin(x) = sin;
///     log(x, b) = log;
///     add(a, b) = add_two_number;
/// }
/// ```
macro_rules! export {
    [ @args $($arg:ident),* $(,)? ] => {
        vec![$(stringify!($arg)),*]
    };
    { @member } => { vec![] };
    { @member $ename:tt = $ntype:ident($value:expr); $($export:tt)* } => {{
        let mut module = vec![ModMember::Var((stringify!($ename), Number::$ntype($value)))];
        module.append(&mut export!{ @member $($export)* });
        module
    }};
    { @member $ename:tt($($para:ident),* $(,)?) = $rust_fn:expr; $($export:tt)* } => {{
        let mut module = vec![ModMember::Fun((stringify!($ename), export![ @args $($para),* ], $rust_fn))];
        module.append(&mut export!{ @member $($export)* });
        module
    }};
    { @member $($invalid:tt)* } => {
        compile_error!(concat!("export! grammer error: possibly missing semicolon => ", stringify!($($invalid)*)));
    };
    { $($export:tt)* } => {
        #[unsafe(no_mangle)]
        pub fn export_module() -> RMExport {
            export!{ @member $($export)* }
        }
    };
}

/// Rust library module member
///
/// Normally, you don't have to construct this enum by yourself, use [`export!`] instead.
pub enum ModMember {
    Var((&'static str, Number)),
    Fun(
        (
            &'static str,
            Vec<&'static str>,
            fn(ScopeApi) -> RMFunRetType,
        ),
    ),
}

/// Enum type to interact with Rust Module API
#[derive(Clone, Copy, Debug)]
pub enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// Rust Module API Type to interact with math-lang runtime
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub enum RMApiType {
    I32,
    I64,
    F64,
    BigNum,
    Sequence,
    ByteArray,
}

/// Informations about a piece of math-lang runtime heap memory
///
/// You can use `VarApi::from()` to construct a math-lang variable with type of [`RMApiType::Sequence`].
/// Checkout [`ScopeApi::allocate()`] for exmaple.
#[derive(Clone, Copy, Debug)]
pub struct RMHeapInfo {
    pub sindex: usize, // scope index
    pub sid: usize,    // scope ID
    pub mstart: usize, // memory start position
    pub mend: usize,   // memory end position
}

/// API to interact with math-lang runtime [`crate::runtime::Scope`]
#[derive(Debug)]
pub struct ScopeApi<'runtime, 'call> {
    builtin: &'call mut Scope<'runtime>,
    locals: &'call mut Vec<Scope<'runtime>>,
}

/// API to interact with math-lang variables
#[derive(Clone, Debug)]
pub struct VarApi {
    rref: Rc<RefCell<Var>>,
}

/// API to interact with math-lang functions
#[derive(Clone, Debug)]
pub struct FunApi<'runtime> {
    rref: Rc<RefCell<Fun<'runtime>>>,
}

impl<'runtime, 'call> ScopeApi<'runtime, 'call> {
    /// Construct a new ScopeApi, only calls by math-lang runtime.
    pub fn new(builtin: &'call mut Scope<'runtime>, locals: &'call mut Vec<Scope<'runtime>>) -> Self
    where
        'runtime: 'call,
    {
        Self { builtin, locals }
    }
    /// Set runtime variable to value by name.
    /// ```
    /// pub fn example(sapi: ScopeApi) -> RMFunRetType {
    ///     let lucky = VarApi::from(0923);
    ///     sapi.set_var("LUCKY", &lucky);
    ///     ...
    /// }
    /// ```
    pub fn set_var(&mut self, name: &'runtime str, vapi: &VarApi) -> VarApi {
        let mut last_name = "";
        for scope in self.locals.iter().rev() {
            if last_name == scope.get_name() {
                continue;
            }
            if let Some(var) = scope.get_var(name) {
                *var.borrow_mut() = Var::clone(&vapi.rref.borrow());
                return VarApi::new(&var);
            }
            last_name = scope.get_name();
        }
        self.locals
            .last_mut()
            .unwrap()
            .add_var(name, Var::clone(&vapi.rref.borrow()));
        VarApi::new(&self.locals.last().unwrap().get_var(name).unwrap())
    }
    /// Set runtime variable to value only in current scope by name.
    pub fn set_current_var(&mut self, name: &'runtime str, vapi: &VarApi) -> VarApi {
        let scope = self.locals.last_mut().unwrap();
        scope.add_var(name, Var::clone(&vapi.rref.borrow()));
        VarApi::new(&scope.get_var(name).unwrap())
    }
    /// Get runtime variable by name excluding builtins.
    /// ```
    /// pub fn example(sapi: ScopeApi) -> RMFunRetType {
    ///     let lucky: VarApi = sapi.get_var("LUCKY").unwrap_or(VarApi::from(0923));
    ///     ...
    /// }
    /// ```
    pub fn get_var(&self, name: &'runtime str) -> Option<VarApi> {
        let mut last_name = "";
        for scope in self.locals.iter().rev() {
            if last_name == scope.get_name() {
                continue;
            }
            if let Some(var) = scope.get_var(name) {
                return Some(VarApi::new(&var));
            }
            last_name = scope.get_name();
        }
        None
    }
    /// Get runtime variable only in current scope by name.
    pub fn get_current_var(&self, name: &'runtime str) -> Option<VarApi> {
        let scope = self.locals.last().unwrap();
        scope.get_var(name).map(|var| VarApi::new(&var))
    }
    /// Get runtime builtin variable by name.
    /// ```
    /// pub fn example(sapi: ScopeApi) -> RMFunRetType {
    ///     let pi: VarApi = sapi.get_builtin_var("pi").unwrap();  // "pi" is builtin constant
    ///     let pi: f64 = pi.try_into().unwrap();      // "pi" is f64 constant
    ///     ...
    /// }
    /// ```
    pub fn get_builtin_var(&self, name: &'runtime str) -> Option<VarApi> {
        self.builtin.get_var(name).map(|var| VarApi::new(&var))
    }
    /// Get runtime function by name excluding builtins.
    pub fn get_fun(&self, name: &'runtime str) -> Option<FunApi<'runtime>> {
        let mut last_name = "";
        for scope in self.locals.iter().rev() {
            if last_name == scope.get_name() {
                continue;
            }
            if let Some(fun) = scope.get_fun(name) {
                return Some(FunApi::new(&fun));
            }
            last_name = scope.get_name();
        }
        None
    }
    /// Get runtime function only in current scope by name.
    pub fn get_current_fun(&self, name: &'runtime str) -> Option<FunApi<'runtime>> {
        let scope = self.locals.last().unwrap();
        scope.get_fun(name).map(|fun| FunApi::new(&fun))
    }
    /// Get runtime builtin function by name.
    pub fn get_builtin_fun(&self, name: &'runtime str) -> Option<FunApi<'runtime>> {
        self.builtin.get_fun(name).map(|fun| FunApi::new(&fun))
    }
    /// Allocate a piece of runtime heap memory.
    /// ```
    /// pub fn example(sapi: ScopeApi) -> RMFunRetType {
    ///     let memory = sapi.allocate(5);
    ///     let seq = VarApi::from(memory);  // Sequence with length 5
    ///     ...
    /// }
    /// ```
    pub fn allocate(&mut self, len: usize) -> RMHeapInfo {
        let sindex = self.locals.len() - 1;
        let sid = self.locals[sindex].get_id();
        let (mstart, mend) = self.locals[sindex].add_arr(len);
        RMHeapInfo {
            sindex,
            sid,
            mstart,
            mend,
        }
    }
    /// Compare wether two runtime variables have the same value.
    pub fn var_eq(&self, lhs: &VarApi, rhs: &VarApi) -> bool {
        let lhsb = lhs.rref.borrow();
        let rhsb = rhs.rref.borrow();
        if lhsb.type_ != rhsb.type_ {
            return false;
        }
        if lhsb.type_ == rhsb.type_ && lhsb.type_ != VarType::Sequence {
            lhsb.as_raw_bytes() == rhsb.as_raw_bytes()
        } else {
            let lheap = lhs.get_heap_info().unwrap();
            let rheap = rhs.get_heap_info().unwrap();
            for (li, ri) in (lheap.mstart..lheap.mend).zip(rheap.mstart..rheap.mend) {
                let l = VarApi::new(&self.locals[lheap.sindex].get_heap(li));
                let r = VarApi::new(&self.locals[rheap.sindex].get_heap(ri));
                if !self.var_eq(&l, &r) {
                    return false;
                }
            }
            true
        }
    }
}

impl VarApi {
    /// Construct a new VarApi, only calls by math-lang runtime.
    pub fn new(ref_var: &Rc<RefCell<Var>>) -> Self {
        Self {
            rref: Rc::clone(ref_var),
        }
    }
    /// Construct an empty [`RMApiType::ByteArray`] variable.
    pub fn none() -> Self {
        Self {
            rref: Rc::new(RefCell::new(Var::none())),
        }
    }
    /// Get inner reference of variable, only calls by math-lang runtime.
    pub fn into_innter(self) -> Rc<RefCell<Var>> {
        self.rref
    }
    /// Set variable to value.
    /// ```
    /// pub fn example(sapi: ScopeApi) -> RMFunRetType {
    ///     let mut lucky: VarApi = sapi.get_var("LUCKY").unwrap_or(VarApi::none());
    ///     lucky.set(Number::U8(0923));
    ///     ...
    /// }
    /// ```
    pub fn set(&mut self, value: Number) {
        match value {
            Number::U8(num) => *self.rref.borrow_mut() = Var::from(num as i32),
            Number::U16(num) => *self.rref.borrow_mut() = Var::from(num as i32),
            Number::U32(num) => *self.rref.borrow_mut() = Var::from(num as i64),
            Number::I8(num) => *self.rref.borrow_mut() = Var::from(num as i32),
            Number::I16(num) => *self.rref.borrow_mut() = Var::from(num as i32),
            Number::I32(num) => *self.rref.borrow_mut() = Var::from(num),
            Number::I64(num) => *self.rref.borrow_mut() = Var::from(num),
            Number::F32(num) => *self.rref.borrow_mut() = Var::from(num as f64),
            Number::F64(num) => *self.rref.borrow_mut() = Var::from(num),
        }
    }
    /// Get the [`RMApiType`] of variable.
    pub fn vtype(&self) -> RMApiType {
        match self.rref.borrow().type_ {
            VarType::I32 => RMApiType::I32,
            VarType::I64 => RMApiType::I64,
            VarType::F64 => RMApiType::F64,
            VarType::BigNum => RMApiType::BigNum,
            VarType::Sequence => RMApiType::Sequence,
            VarType::None => RMApiType::ByteArray,
        }
    }
    /// Set variable with raw bytes.
    ///
    /// This function will let the variable become [`RMApiType::ByteArray`] for the safety of runtime.
    /// ```
    /// pub fn example(sapi: ScopeApi) -> RMFunRetType {
    ///     let mut name_api: VarApi = sapi.get_current_var("name").unwrap();
    ///     let name = "Chisato";
    ///     nam_api.set_bytes(name.as_bytes());
    ///     ...
    /// }
    /// ```
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        let mut mref = self.rref.borrow_mut();
        mref.type_ = VarType::None;
        mref.write_data_unchecked(bytes);
    }
    /// Get runtime heap memory info.
    ///
    /// Return actual type if variable is not [`RMApiType::Sequence`].
    /// ```
    /// pub fn example(sapi: ScopeApi) -> RMFunRetType {
    ///     let seq = sapi.get_current_var("seq").unwrap();
    ///     let memory = seq.get_heap_info().unwrap();
    ///     let same_seq = VarApi::from(memory);  // VarApi owns the same runtime memory (sequence) with seq
    ///     assert!(sapi.var_eq(seq, same_seq));
    ///     ...
    /// }
    /// ```
    pub fn get_heap_info(&self) -> Result<RMHeapInfo, RMApiType> {
        let var = self.rref.borrow();
        if var.type_ != VarType::Sequence {
            return Err(self.vtype());
        }
        let (sindex, sid) = var.get_scope();
        let (mstart, mend) = var.get_boundary();
        Ok(RMHeapInfo {
            sindex,
            sid,
            mstart,
            mend,
        })
    }
}

macro_rules! impl_var_api_try_into {
    ($rtype:tt, $vtype:ident) => {
        impl TryInto<$rtype> for VarApi {
            type Error = RMApiType;
            fn try_into(self) -> Result<$rtype, Self::Error> {
                let vtype = self.vtype();
                if vtype <= RMApiType::$vtype && vtype <= RMApiType::BigNum {
                    let v: $rtype = (&*self.rref.borrow()).into();
                    Ok(v)
                } else {
                    Err(self.vtype())
                }
            }
        }
    };
}
impl_var_api_try_into!(i32, I32);
impl_var_api_try_into!(i64, I64);
impl_var_api_try_into!(f64, F64);

macro_rules! impl_var_api_try_into_other {
    ($from:tt, $to:tt) => {
        impl TryInto<$to> for VarApi {
            type Error = RMApiType;
            fn try_into(self) -> Result<$to, Self::Error> {
                let vtype = self.vtype();
                let value: $from = self.try_into()?;
                Ok($to::try_from(value).map_err(|_| vtype)?)
            }
        }
    };
}
impl_var_api_try_into_other!(i64, u8);
impl_var_api_try_into_other!(i64, u16);
impl_var_api_try_into_other!(i64, i8);
impl_var_api_try_into_other!(i64, i16);
impl_var_api_try_into_other!(i64, u32);
impl_var_api_try_into_other!(i64, u64);

macro_rules! impl_var_api_from {
    ($rtype:tt) => {
        impl From<$rtype> for VarApi {
            fn from(value: $rtype) -> Self {
                let ref_var = Rc::new(RefCell::new(Var::from(value)));
                Self { rref: ref_var }
            }
        }
    };
}
impl_var_api_from!(i32);
impl_var_api_from!(i64);
impl_var_api_from!(f64);

macro_rules! impl_var_api_from_other {
    ($from:tt, $to:tt) => {
        impl From<$from> for VarApi {
            fn from(value: $from) -> Self {
                VarApi::from(value as $to)
            }
        }
    };
}
impl_var_api_from_other!(f32, f64);
impl_var_api_from_other!(i8, i32);
impl_var_api_from_other!(i16, i32);
impl_var_api_from_other!(u8, i32);
impl_var_api_from_other!(u16, i32);
impl_var_api_from_other!(u32, i64);

impl From<&str> for VarApi {
    fn from(value: &str) -> Self {
        let mut var = Var::default();
        var.write_data_unchecked(value.as_bytes());
        Self {
            rref: Rc::new(RefCell::new(var)),
        }
    }
}

impl TryInto<String> for VarApi {
    type Error = RMApiType;
    fn try_into(self) -> Result<String, Self::Error> {
        if self.vtype() == RMApiType::ByteArray {
            Ok(self.rref.borrow().to_string())
        } else {
            Err(self.vtype())
        }
    }
}

impl TryInto<Vec<u8>> for VarApi {
    type Error = RMApiType;
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        if self.vtype() == RMApiType::ByteArray {
            Ok(self.rref.borrow().as_raw_bytes().to_vec())
        } else {
            Err(self.vtype())
        }
    }
}

impl From<RMHeapInfo> for VarApi {
    fn from(value: RMHeapInfo) -> Self {
        let var = Var::new_sequence((value.mstart, value.mend), (value.sindex, value.sid));
        Self {
            rref: Rc::new(RefCell::new(var)),
        }
    }
}

impl VarApi {
    /// Convert into [`Vec<VarApi>`] if the variable is [`RMApiType::Sequence`]
    ///
    /// Return actual type if variable is not [`RMApiType::Sequence`].
    /// ```
    /// pub fn example(sapi: ScopeApi) -> RMFunRetType {
    ///     let mut name_vec = sapi
    ///         .get_current_var("name_seq")
    ///         .unwrap()
    ///         .try_into_sequence()
    ///         .map_err(|t| format!("expect Sequence got {}", t))?;
    ///
    ///     name_vec[0].set_bytes(b"Alice");
    ///     name_vec[1].set_bytes(b"Bob");
    ///     ...
    /// }
    /// ```
    pub fn try_into_sequence(self, sapi: &ScopeApi) -> Result<Vec<VarApi>, RMApiType> {
        if self.rref.borrow().type_ != VarType::Sequence {
            return Err(self.vtype());
        }
        let bd = self.rref.borrow().get_boundary();
        let (sc_index, sc_id) = self.rref.borrow().get_scope();
        let scope = &sapi.locals[sc_index];
        assert_eq!(
            scope.get_id(),
            sc_id,
            "Rust Module API: custom module internal error!"
        );
        let mut seq = Vec::with_capacity(bd.1 - bd.0);
        for i in bd.0..bd.1 {
            let mem = scope.get_heap(i);
            seq.push(VarApi::new(&mem));
        }
        Ok(seq)
    }
}

impl<'runtime> FunApi<'runtime> {
    /// Construct a new FunApi, only calls by math-lang runtime.
    pub fn new(ref_fun: &Rc<RefCell<Fun<'runtime>>>) -> Self {
        Self {
            rref: Rc::clone(ref_fun),
        }
    }
}

impl std::fmt::Display for RMApiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RMApiType::I32 => "I32",
                RMApiType::I64 => "I64",
                RMApiType::F64 => "F64",
                RMApiType::BigNum => "BigNum",
                RMApiType::Sequence => "Sequence",
                RMApiType::ByteArray => "ByteArray",
            }
        )
    }
}
