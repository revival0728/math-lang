use crate::big_num::BigNum;
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
    Real,
    Sequence,
}

#[derive(Debug, Default, Clone, Eq)]
pub struct Var {
    pub type_: VarType,
    data: Vec<u8>,
}

impl PartialEq for Var {
    fn eq(&self, other: &Self) -> bool {
        let tp = std::cmp::max(self.type_, other.type_);
        macro_rules! primitive_eq {
            ($ptype:ident) => {{
                let l: $ptype = self.into();
                let r: $ptype = other.into();
                l == r
            }};
        }
        match tp {
            VarType::None => self.data == other.data,
            VarType::I32 => primitive_eq!(i32),
            VarType::I64 => primitive_eq!(i64),
            VarType::Real => {
                BigNum::from_le_bytes(&self.data) == BigNum::from_le_bytes(&other.data)
            }
            VarType::Sequence => self.data == other.data,
        }
    }
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
                Self::Real => "Real",
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
        if let Ok(parsed) = BigNum::try_from(value) {
            let bytes = parsed.to_le_bytes();
            return Some(Self {
                type_: VarType::Real,
                data: bytes,
            });
        }
        None
    }
    pub fn write_data_unchecked(&mut self, data: &[u8]) {
        self.data = data.to_vec();
    }
    pub fn as_raw_bytes(&self) -> &[u8] {
        self.data.as_slice()
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
impl_from_for_var!(BigNum, Real);

impl Into<i32> for &Var {
    fn into(self) -> i32 {
        match self.type_ {
            VarType::None => panic!("runtime internal error!"),
            VarType::I32 => i32::from_le_bytes(self.data[0..4].try_into().unwrap()).into(),
            _ => panic!("runtime internal error!"),
        }
    }
}

impl Into<i64> for &Var {
    fn into(self) -> i64 {
        match self.type_ {
            VarType::None => panic!("runtime internal error!"),
            VarType::I32 => i32::from_le_bytes(self.data[0..4].try_into().unwrap()).into(),
            VarType::I64 => i64::from_le_bytes(self.data[0..8].try_into().unwrap()).into(),
            _ => panic!("runtime internal error!"),
        }
    }
}

impl Into<BigNum> for &Var {
    fn into(self) -> BigNum {
        match self.type_ {
            VarType::None => panic!("runtime internal error!"),
            VarType::I32 => BigNum::from(i32::from_le_bytes(self.data[0..4].try_into().unwrap())),
            VarType::I64 => BigNum::from(i64::from_le_bytes(self.data[0..8].try_into().unwrap())),
            VarType::Real => BigNum::from_le_bytes(&self.data),
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
            VarType::I32 | VarType::I64 | VarType::Real => true,
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
                match std::cmp::max(VarType::$min_type, std::cmp::max(self.type_, rhs.type_)) {
                    VarType::None | VarType::Sequence => panic!("runtime internal error!"),
                    VarType::I32 => {
                        let l: i32 = self.into();
                        let r: i32 = rhs.into();
                        if let Some(v) = l.$checked_fn(r) {
                            Var::from(v)
                        } else {
                            let l: i64 = self.into();
                            let r: i64 = rhs.into();
                            Var::from(l.$uncheck_fn(r))
                        }
                        
                    },
                    VarType::I64 => {
                        let l: i64 = self.into();
                        let r: i64 = rhs.into();
                        if let Some(v) = l.$checked_fn(r) {
                            Var::from(v)
                        } else {
                            let l: BigNum = self.into();
                            let r: BigNum = rhs.into();
                            Var::from(l.$uncheck_fn(&r))
                        }
                    },
                    VarType::Real => {
                        let l: BigNum = self.into();
                        let r: BigNum = rhs.into();
                        let mut res = l.$uncheck_fn(&r);
                        res.trunc_with_precision(180);
                        Var::from(res)
                    }
                }
            }
        }
    };
}
impl_operation_for_var!(Add, add, checked_add, add, None);
impl_operation_for_var!(Sub, sub, checked_sub, sub, None);
impl_operation_for_var!(Mul, mul, checked_mul, mul, None);
impl_operation_for_var!(Div, div, checked_div, div, Real);
impl_operation_for_var!(Rem, rem, checked_rem_euclid, rem_euclid, I64); // start from I64 to handle rhs == 0

// TODO: optimized implementation
impl Neg for &Var {
    type Output = Var;
    fn neg(self) -> Self::Output {
        match self.type_ {
            VarType::None => panic!("runtime internal error!"),
            VarType::I32 => {
                let v: i32 = self.into();
                if let Some(v) = v.checked_neg() {
                    Var::from(v)
                } else {
                    let v: i64 = self.into();
                    Var::from(-v)
                }
            }
            VarType::I64 => {
                let v: i64 = self.into();
                if let Some(v) = v.checked_neg() {
                    Var::from(v)
                } else {
                    let v: BigNum = self.into();
                    Var::from(-&v)
                }
            }
            VarType::Real => {
                let v: BigNum = self.into();
                Var::from(-&v)
            }
            VarType::Sequence => panic!("runtime internal error!"),
        }
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.type_ {
            VarType::None => {
                if self.data.is_empty() {
                    write!(f, "")
                } else {
                    write!(
                        f,
                        "{}",
                        match str::from_utf8(&self.data) {
                            Ok(s) => s,
                            Err(_) =>
                                "<#!this None type is not a string literal and cannot be displayed!#>",
                        }
                    )
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
            VarType::Real => {
                let v: BigNum = self.into();
                // SAFE: no multiple threads
                unsafe {
                    let p = PRECISION as usize;
                    write!(f, "{}", v.to_float_str(p as u8))
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
