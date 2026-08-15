// File for Rust Module API
//
// Rust Module define example
//
// pub fn rust_add_i32(sapi: ScopeApi) -> RMFunRetType {
//     let a: i32 = sapi.get_current_var("a").unwrap().try_into().map_err(|t| format!("expected I32 type got {} type", t))?;
//     let b: i32 = sapi.get_current_var("b").unwrap().try_into().map_err(|t| format!("expected I32 type got {} type", t))?;
//     let result = a + b;
//     Ok(Some(VarApi::from(result)))
// }
// export! {
//     pi = F64(3.14);
//     add(a, b) = rust_add_i32;
// }
//
// For more examples please check out src/builtin.rs

#![allow(unused)]
use crate::runtime::{Fun, Scope};
use crate::var::{Var, VarType};
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

pub type RMFunResult<T> = Result<T, String>;
pub type RMFunRetType = RMFunResult<Option<VarApi>>;
pub type RMFunPtr = fn(ScopeApi) -> RMFunRetType;
pub type RMExport = Vec<ModMember>;

#[macro_export]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub enum RMApiType {
    I32,
    I64,
    F64,
    BigNum,
    Sequence,
    ByteArray,
}

#[derive(Clone, Copy, Debug)]
pub struct RMHeapInfo {
    pub sindex: usize,
    pub sid: usize,
    pub mstart: usize,
    pub mend: usize,
}

#[derive(Debug)]
pub struct ScopeApi<'runtime, 'call> {
    builtin: &'call mut Scope<'runtime>,
    locals: &'call mut Vec<Scope<'runtime>>,
}

#[derive(Clone, Debug)]
pub struct VarApi {
    rref: Rc<RefCell<Var>>,
}

#[derive(Clone, Debug)]
pub struct FunApi<'runtime> {
    rref: Rc<RefCell<Fun<'runtime>>>,
}

impl<'runtime, 'call> ScopeApi<'runtime, 'call> {
    pub fn new(builtin: &'call mut Scope<'runtime>, locals: &'call mut Vec<Scope<'runtime>>) -> Self
    where
        'runtime: 'call,
    {
        Self { builtin, locals }
    }
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
    pub fn set_current_var(&mut self, name: &'runtime str, vapi: &VarApi) -> VarApi {
        let scope = self.locals.last_mut().unwrap();
        scope.add_var(name, Var::clone(&vapi.rref.borrow()));
        VarApi::new(&scope.get_var(name).unwrap())
    }
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
    pub fn get_current_var(&self, name: &'runtime str) -> Option<VarApi> {
        let scope = self.locals.last().unwrap();
        scope.get_var(name).map(|var| VarApi::new(&var))
    }
    pub fn get_builtin_var(&self, name: &'runtime str) -> Option<VarApi> {
        self.builtin.get_var(name).map(|var| VarApi::new(&var))
    }
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
    pub fn get_current_fun(&self, name: &'runtime str) -> Option<FunApi<'runtime>> {
        let scope = self.locals.last().unwrap();
        scope.get_fun(name).map(|fun| FunApi::new(&fun))
    }
    pub fn get_builtin_fun(&self, name: &'runtime str) -> Option<FunApi<'runtime>> {
        self.builtin.get_fun(name).map(|fun| FunApi::new(&fun))
    }
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
    pub fn new(ref_var: &Rc<RefCell<Var>>) -> Self {
        Self {
            rref: Rc::clone(ref_var),
        }
    }
    pub fn none() -> Self {
        Self {
            rref: Rc::new(RefCell::new(Var::none())),
        }
    }
    pub fn into_innter(self) -> Rc<RefCell<Var>> {
        self.rref
    }
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
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        let mut mref = self.rref.borrow_mut();
        mref.type_ = VarType::None;
        mref.write_data_unchecked(bytes);
    }
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
    pub fn try_into_sequence(self, sapi: &ScopeApi) -> Option<Vec<VarApi>> {
        if self.rref.borrow().type_ != VarType::Sequence {
            return None;
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
        Some(seq)
    }
}

impl<'runtime> FunApi<'runtime> {
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
