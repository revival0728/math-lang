use crate::comiler::{Compiler, CompilerError, Inst};
use std::collections::HashMap;
use std::convert::{From, Into};
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum VarType {
    #[default]
    None,
    I32,
    I64,
    F64,
    BigNum,
}

#[derive(Debug, Default, Clone)]
pub struct Var {
    type_: VarType,
    data: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct Fun<'input> {
    data: Vec<Inst<'input>>,
}

#[derive(Debug, Default, Clone)]
pub struct RuntimeError {
    line: usize,
    msg: String,
}

#[derive(Debug, Default, Clone)]
pub enum GlobalError {
    #[default]
    None,
    RE(RuntimeError),
    CE(CompilerError),
}

#[derive(Debug, Default, Clone)]
pub struct Scope<'input> {
    var_table: HashMap<&'input str, Var>,
    fun_table: HashMap<&'input str, Fun<'input>>,
}

#[derive(Debug, Default, Clone)]
pub struct Runtime<'input> {
    source: &'input str,
    global: Scope<'input>,
    output: Vec<String>,
}

impl<'input> Var {
    pub fn new(value: &'input str) -> Option<Self> {
        macro_rules! try_parse {
            ($rust_type:ident, $var_type:ident, $bytes_of_type:literal) => {
                if let Ok(parsed) = value.parse::<$rust_type>() {
                    let bytes = parsed.to_le_bytes();
                    let data = Vec::from(bytes);
                    return Some(Self {
                        type_: VarType::$var_type,
                        data,
                    });
                }
            };
        }
        try_parse!(i32, I32, 4);
        try_parse!(i64, I64, 8);
        try_parse!(f64, F64, 8);
        // TODO: implement BigNum
        None
    }
}

macro_rules! impl_from_for_var {
    ($rust_type:ident, $var_type:ident) => {
        impl From<$rust_type> for Var {
            fn from(value: $rust_type) -> Self {
                Self {
                    type_: VarType::$var_type,
                    data: Vec::from(value.to_le_bytes()),
                }
            }
        }
    };
}
impl_from_for_var!(i32, I32);
impl_from_for_var!(i64, I64);
impl_from_for_var!(f64, F64);

macro_rules! impl_into_for_var {
    ($rust_type:ident, $bytes_of_type:literal) => {
        impl Into<$rust_type> for &Var {
            fn into(self) -> $rust_type {
                $rust_type::from_le_bytes(self.data[0..$bytes_of_type].try_into().unwrap())
            }
        }
    };
}
impl_into_for_var!(i32, 4);
impl_into_for_var!(i64, 8);
impl_into_for_var!(f64, 8);

// TODO: fixed f64, i64 higher type problem
macro_rules! impl_operation_for_var {
    ($oper:ident, $fname:ident, $checked_fn:ident) => {
        impl $oper<&Var> for &Var {
            type Output = Var;
            fn $fname(self, rhs: &Var) -> Self::Output {
                macro_rules! impl_integer {
                    ($ltype:ident, $utype:ident) => {{
                        let l: $ltype = self.into();
                        let r: $ltype = self.into();
                        if let Some(v) = l.$checked_fn(r) {
                            Var::from(v)
                        } else {
                            let l: $utype = l.into();
                            let r: $utype = r.into();
                            Var::from(l + r)
                        }
                    }};
                }
                match std::cmp::max(self.type_, rhs.type_) {
                    VarType::None | VarType::BigNum => panic!("runtime internal error!"),
                    VarType::I32 => impl_integer!(i32, i64),
                    VarType::I64 => impl_integer!(i64, i64),
                    VarType::F64 => {
                        let l: f64 = self.into();
                        let r: f64 = rhs.into();
                        Var::from(l.$fname(r))
                    }
                }
            }
        }
    };
}
impl_operation_for_var!(Add, add, checked_add);
impl_operation_for_var!(Sub, sub, checked_sub);
impl_operation_for_var!(Mul, mul, checked_mul);
impl_operation_for_var!(Div, div, checked_div);

// TODO: implement BigNum display
impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.type_ {
            VarType::None | VarType::BigNum => panic!("runtime internal error!"),
            VarType::I32 => {
                let v: i32 = self.into();
                write!(f, "{}", v)
            }
            VarType::I64 => {
                let v: i64 = self.into();
                write!(f, "{}", v)
            }
            VarType::F64 => {
                let v: f64 = self.into();
                write!(f, "{:.05}", v)
            }
        }
    }
}

impl<'input> Scope<'input> {
    pub fn get_var(&self, name: &'input str) -> Option<&Var> {
        self.var_table.get(name)
    }
    pub fn get_var_mut(&mut self, name: &'input str) -> Option<&mut Var> {
        self.var_table.get_mut(name)
    }
    pub fn get_fun(&self, name: &'input str) -> Option<&Fun<'input>> {
        self.fun_table.get(name)
    }
    pub fn get_fun_mut(&mut self, name: &'input str) -> Option<&mut Fun<'input>> {
        self.fun_table.get_mut(name)
    }
}

impl<'input> Runtime<'input> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn execute(source: &'input str) -> Result<&Vec<String>, GlobalError> {
        todo!()
    }
}
