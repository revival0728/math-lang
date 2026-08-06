// File for Rust Module API
//
// Rust Module define example
//
// pub fn rust_add_i32(sapi: ScopeApi) -> Option<VarApi> {
//     let a: i32 = sapi.get_current_var("a").unwrap().try_into().unwrap();
//     let b: i32 = sapi.get_current_var("b").unwrap().try_into().unwrap();
//     let result = a + b;
//     Some(VarApi::from(result))
// }
// export! {
//     pi = F64(3.14);
//     add(a, b) = rust_add_i32;
// }

use crate::runtime::Fun;
use crate::var::VarType;
use crate::{runtime::Scope, var::Var};
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

pub enum ModMember {
    Var((&'static str, Number)),
    Fun(
        (
            &'static str,
            Vec<&'static str>,
            fn(ScopeApi) -> Option<VarApi>,
        ),
    ),
}

macro_rules! parse_func_args {
    [ $($arg:ident),* $(,)? ] => {
        vec![ $( stringify!($arg) ),* ]
    };
}

macro_rules! export_member {
    {} => { vec![] };
    { $ename:ident = $ntype:ident($value:expr); $($export:tt)* } => {{
        let mut module = vec![ModMember::Var((stringify!($ename), Number::$ntype($value)))];
        module.append(&mut export_member!{ $($export)* });
        module
    }};
    { $ename:ident($($para:ident),* $(,)?) = $rust_fn:expr; $($export:tt)* } => {{
        let mut module = vec![ModMember::Fun((stringify!($ename), parse_func_args![$($para),*], $rust_fn))];
        module.append(&mut export_member!{ $($export)* });
        module
    }};
}

#[macro_export]
macro_rules! export {
    { $($export:tt)* } => {
        pub fn export_module() -> Vec<ModMember> {
            export_member!{ $($export)* }
        }
    };
}

pub type RMApiResult<T> = Result<T, ()>;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RMApiType {
    I32,
    I64,
    F64,
    BigNum,
    Sequence,
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
}

impl VarApi {
    pub fn new(ref_var: &Rc<RefCell<Var>>) -> Self {
        Self {
            rref: Rc::clone(ref_var),
        }
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
            VarType::None => panic!("Rust Module API: custom module internal error!"),
            VarType::I32 => RMApiType::I32,
            VarType::I64 => RMApiType::I64,
            VarType::F64 => RMApiType::F64,
            VarType::BigNum => RMApiType::BigNum,
            VarType::Sequence => RMApiType::Sequence,
        }
    }
}

macro_rules! impl_var_api_try_into {
    ($rtype:tt, $vtype:ident) => {
        impl TryInto<$rtype> for VarApi {
            type Error = ();
            fn try_into(self) -> Result<$rtype, Self::Error> {
                if self.rref.borrow().type_ == VarType::$vtype {
                    let v: $rtype = (&*self.rref.borrow()).into();
                    Ok(v)
                } else {
                    Err(())
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
            type Error = ();
            fn try_into(self) -> Result<$to, Self::Error> {
                let value: $from = self.try_into()?;
                Ok(value as $to)
            }
        }
    };
}

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
