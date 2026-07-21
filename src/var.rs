use crate::env::PRECISION;
use std::convert::{From, Into};
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

// TODO: write tests for struct Var

#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum VarType {
    #[default]
    None,
    I32,
    I64,
    F64,
    BigNum,
    Sequence,
}

#[derive(Debug, Default, Clone)]
pub struct Var {
    pub type_: VarType,
    data: Vec<u8>,
}

impl Display for VarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "None",
                Self::I32 => "I32",
                Self::I64 => "I64",
                Self::F64 => "F64",
                Self::BigNum => "BigNum",
                Self::Sequence => "Sequence",
            }
        )
    }
}

impl<'input> Var {
    pub fn none() -> Self {
        Self::default()
    }
    pub fn from_string(s: String) -> Self {
        let data = s.into_bytes();
        Var {
            type_: VarType::None,
            data,
        }
    }
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

impl Into<i32> for &Var {
    fn into(self) -> i32 {
        match self.type_ {
            VarType::BigNum => panic!("need BigNum implementation"),
            VarType::None => panic!("runtime internal error!"),
            VarType::I32 => i32::from_le_bytes(self.data[0..4].try_into().unwrap()).into(),
            _ => panic!("runtime internal error!"),
        }
    }
}

impl Into<i64> for &Var {
    fn into(self) -> i64 {
        match self.type_ {
            VarType::BigNum => panic!("need BigNum implementation"),
            VarType::None => panic!("runtime internal error!"),
            VarType::I32 => i32::from_le_bytes(self.data[0..4].try_into().unwrap()).into(),
            VarType::I64 => i64::from_le_bytes(self.data[0..8].try_into().unwrap()).into(),
            _ => panic!("runtime internal error!"),
        }
    }
}

// FIXME: i64 to f64 may cause overflow, change to BigNum instead
impl Into<f64> for &Var {
    fn into(self) -> f64 {
        match self.type_ {
            VarType::BigNum => panic!("need BigNum implementation"),
            VarType::None => panic!("runtime internal error!"),
            VarType::I32 => i32::from_le_bytes(self.data[0..4].try_into().unwrap()).into(),
            VarType::I64 => i64::from_le_bytes(self.data[0..8].try_into().unwrap()) as f64,
            VarType::F64 => f64::from_le_bytes(self.data[0..8].try_into().unwrap()).into(),
            VarType::Sequence => panic!("runtime internal error!"),
        }
    }
}

// for sequence
impl Var {
    pub fn new_sequence(ptr: (usize, usize), scope: (usize, usize)) -> Self {
        let start = ptr.0.to_le_bytes();
        let end = ptr.1.to_le_bytes();
        let scope_index = scope.0.to_le_bytes();
        let scope_id = scope.1.to_le_bytes();
        let mut data = Vec::new();
        data.extend_from_slice(&start);
        data.extend_from_slice(&end);
        data.extend_from_slice(&scope_index);
        data.extend_from_slice(&scope_id);
        Self {
            type_: VarType::Sequence,
            data,
        }
    }
    pub fn is_num(&self) -> bool {
        match self.type_ {
            VarType::None => panic!("runtime internal error!"),
            VarType::Sequence => false,
            VarType::I32 | VarType::I64 | VarType::F64 | VarType::BigNum => true,
        }
    }
    pub fn get_scope(&self) -> (usize, usize) {
        let info: ((usize, usize), (usize, usize)) = self.into();
        info.1
    }
    pub fn get_boundary(&self) -> (usize, usize) {
        let info: ((usize, usize), (usize, usize)) = self.into();
        info.0
    }
}

impl Into<((usize, usize), (usize, usize))> for &Var {
    fn into(self) -> ((usize, usize), (usize, usize)) {
        const UB: usize = (usize::BITS >> 3) as usize;
        let start = usize::from_le_bytes(self.data[0..UB].try_into().unwrap());
        let end = usize::from_le_bytes(self.data[UB..(UB << 1)].try_into().unwrap());
        let scope_index =
            usize::from_le_bytes(self.data[(UB << 1)..((UB << 1) + UB)].try_into().unwrap());
        let scope_id =
            usize::from_le_bytes(self.data[((UB << 1) + UB)..(UB << 2)].try_into().unwrap());
        ((start, end), (scope_index, scope_id))
    }
}

// TODO: fixed f64, i64 higher type problem
#[rustfmt::skip]
macro_rules! impl_operation_for_var {
    ($oper:ident, $fname:ident, $checked_fn:ident, $uncheck_fn:ident, $min_type:ident) => {
        impl $oper<&Var> for &Var {
            type Output = Var;
            fn $fname(self, rhs: &Var) -> Self::Output {
                macro_rules! impl_integer {
                    ($ltype:ident, $utype:ident) => {{
                        let l: $ltype = self.into();
                        let r: $ltype = rhs.into();
                        if let Some(v) = l.$checked_fn(r) {
                            Var::from(v)
                        } else {
                            let l: $utype = self.into();
                            let r: $utype = rhs.into();
                            Var::from(l.$uncheck_fn(r))
                        }
                    }};
                }
                match std::cmp::max(VarType::$min_type, std::cmp::max(self.type_, rhs.type_)) {
                    VarType::None | VarType::BigNum | VarType::Sequence => panic!("runtime internal error!"),
                    VarType::I32 => impl_integer!(i32, i64),
                    VarType::I64 => impl_integer!(i64, f64),
                    VarType::F64 => {
                        let l: f64 = self.into();
                        let r: f64 = rhs.into();
                        Var::from(l.$uncheck_fn(r))
                    }
                }
            }
        }
    };
}
impl_operation_for_var!(Add, add, checked_add, add, None);
impl_operation_for_var!(Sub, sub, checked_sub, sub, None);
impl_operation_for_var!(Mul, mul, checked_mul, mul, None);
impl_operation_for_var!(Div, div, checked_div, div, F64);
impl_operation_for_var!(Rem, rem, checked_rem_euclid, rem_euclid, I64); // start from I64 to handle rhs == 0

// TODO: optimized implementation
impl Neg for &Var {
    type Output = Var;
    fn neg(self) -> Self::Output {
        macro_rules! impl_for_integer {
            ($rust_type:ident, $utype:ident) => {{
                let v: $rust_type = self.into();
                if let Some(v) = v.checked_neg() {
                    Var::from(v)
                } else {
                    let v: $utype = self.into();
                    Var::from(-v)
                }
            }};
        }
        macro_rules! impl_for_float {
            ($rust_type:ident) => {{
                let v: $rust_type = self.into();
                Var::from(-v)
            }};
        }
        match self.type_ {
            VarType::None => panic!("runtime internal error!"),
            VarType::I32 => impl_for_integer!(i32, i64),
            VarType::I64 => impl_for_integer!(i64, f64),
            VarType::F64 => impl_for_float!(f64),
            VarType::BigNum => panic!("missing BigNum implementation"),
            VarType::Sequence => panic!("runtime internal error!"),
        }
    }
}

// TODO: implement BigNum display
impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.type_ {
            VarType::BigNum => panic!("runtime internal error!"),
            VarType::None => {
                if self.data.is_empty() {
                    write!(f, "")
                } else {
                    let s = str::from_utf8(&self.data).expect("runtime internal error!");
                    write!(f, "{}", s)
                }
            }
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
                // SAFE: no multiple threads
                unsafe {
                    let p = PRECISION as usize;
                    write!(f, "{:.p$}", v)
                }
            }
            VarType::Sequence => {
                let (start, end) = self.get_boundary();
                let (_scope_index, scope_id) = self.get_scope();
                // FIXME: break on 128bit?
                write!(
                    f,
                    "<Sequence of Scope {} with length {} at {:#018x}>",
                    scope_id,
                    end - start,
                    start
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::var::Var;

    #[test]
    fn sequence() {
        let arr = Var::new_sequence((1, 5), (8, 9));
        let info: ((usize, usize), (usize, usize)) = (&arr).into();
        assert_eq!(info, ((1, 5), (8, 9)));
        assert_eq!(arr.get_boundary(), (1, 5));
        assert_eq!(arr.get_scope(), (8, 9));
    }
}
