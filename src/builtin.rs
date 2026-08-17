#![allow(unused)]
use crate::env::*;
use crate::export;
use crate::rmapi::*;

pub mod consts {
    pub const PI: f64 = std::f64::consts::PI;
    pub const E: f64 = std::f64::consts::E;
}

pub mod math {
    use super::*;
    macro_rules! declare_math_fn {
        ($name:ident $(, $arg:expr)?) => {
            pub fn $name(sapi: ScopeApi) -> RMFunRetType {
                let x: f64 =
                    sapi.get_current_var("x").unwrap().try_into().map_err(|t| {
                        format!("expect argument of {}(x) to be a number but got {}", stringify!($name), t)
                    })?;
                Ok(Some(VarApi::from(x.$name($($arg)?))))
            }
        };
    }
    declare_math_fn!(sin);
    declare_math_fn!(cos);
    declare_math_fn!(tan);
    declare_math_fn!(asin);
    declare_math_fn!(acos);
    declare_math_fn!(atan);
    declare_math_fn!(abs);
    declare_math_fn!(sqrt);
    declare_math_fn!(ceil);
    declare_math_fn!(floor);
    declare_math_fn!(round);
    declare_math_fn!(exp);
    declare_math_fn!(log, 10.0);
    declare_math_fn!(log2);
    declare_math_fn!(ln);
    declare_math_fn!(trunc);
    declare_math_fn!(cbrt);
}

pub mod env {
    use super::*;
    macro_rules! declare_env_fn {
        ($fn:ident, $env:ident, $limit:expr) => {
            pub fn $fn(sapi: ScopeApi) -> RMFunRetType {
                let x: i64 = sapi.get_current_var("x").unwrap().try_into().map_err(|t| {
                    format!(
                        "expect arguemnt of {}(x) to be a integer but got {}",
                        stringify!($env).to_lowercase(),
                        t
                    )
                })?;
                let x: u32 = x.try_into().map_err(|_| {
                    format!(
                        "cannot config {} with negative number or large number",
                        stringify!($env)
                    )
                })?;
                if x > $limit {
                    Err(format!(
                        "env({}) must less than {}",
                        stringify!($env),
                        $limit
                    ))
                } else {
                    unsafe { $env = x };
                    Ok(Some(VarApi::from(unsafe { $env } as i64)))
                }
            }
        };
    }
    declare_env_fn!(env_precision, PRECISION, 15);
    declare_env_fn!(env_print_set_inst, PRINT_SET_INST, 1);
    declare_env_fn!(env_detail_depth, DETAIL_DEPTH, 1);
    declare_env_fn!(env_max_stack_depth, MAX_STACK_DEPTH, u32::MAX);
    declare_env_fn!(env_index_base, INDEX_BASE, 1);
}

pub mod logic {
    use super::*;
    macro_rules! declare_logic_fn {
        ($name:ident, x $logic:tt $value:literal) => {
            pub fn $name(sapi: ScopeApi) -> RMFunRetType {
                let x: i64 = sapi.get_current_var("x").unwrap().try_into().map_err(|t| {
                    format!(
                        "expect argument of {}(x) to be a integer got {}",
                        stringify!($name),
                        t
                    )
                })?;
                Ok(Some(VarApi::from((x $logic $value) as i32)))
            }
        };
    }
    declare_logic_fn!(iff, x == 0);
    declare_logic_fn!(elsef, x != 0);

    pub fn sign(sapi: ScopeApi) -> RMFunRetType {
        let x: f64 = sapi
            .get_current_var("x")
            .unwrap()
            .try_into()
            .map_err(|t| format!("expect argument of sign(x) to be a number but got {}", t))?;
        use std::cmp::Ordering;
        let result = match x.total_cmp(&0.0) {
            Ordering::Equal => 0,
            Ordering::Greater => 1,
            Ordering::Less => -1,
        };
        Ok(Some(VarApi::from(result)))
    }
}

pub mod special {
    use super::*;
    pub fn none(_sapi: ScopeApi) -> RMFunRetType {
        Ok(Some(VarApi::none()))
    }
    pub fn one(_sapi: ScopeApi) -> RMFunRetType {
        Ok(Some(VarApi::from(1)))
    }
}

pub mod mtype {
    use super::*;
    pub fn int32(sapi: ScopeApi) -> RMFunRetType {
        // TODO: change to BigNum when avaliable
        let x: f64 = sapi
            .get_current_var("x")
            .unwrap()
            .try_into()
            .map_err(|t| format!("expect argument of int32(x) to be a number but got {}", t))?;
        let trunc = x.trunc();
        let xi32 = x as i32;
        if trunc == f64::from(xi32) {
            Ok(Some(VarApi::from(xi32)))
        } else {
            Err(format!("cannot convert {} to I32", x))
        }
    }
}

pub mod sequence {
    use super::*;
    pub fn new(mut sapi: ScopeApi) -> RMFunRetType {
        let len: i64 = sapi
            .get_current_var("len")
            .unwrap()
            .try_into()
            .map_err(|t| format!("cannot init a sequence with {}, expect integer", t))?;
        let len: usize = len.try_into().map_err(|_| {
            format!(
                "cannot init a sequence with length {} due to architecture of your computer",
                len
            )
        })?;
        let mem = sapi.allocate(len);
        Ok(Some(VarApi::from(mem)))
    }
    pub fn len(sapi: ScopeApi) -> RMFunRetType {
        let seq = sapi.get_current_var("seq").unwrap();
        let hf = seq
            .get_heap_info()
            .map_err(|t| format!("expect argument of len(seq) to be a sequence but got {}", t))?;
        // FIXME: breaks on 128-bit?
        let len = (hf.mend - hf.mstart) as i64;
        Ok(Some(VarApi::from(len)))
    }
}

pub mod control {
    use super::*;
    pub fn abort(sapi: ScopeApi) -> RMFunRetType {
        let msg: String = sapi
            .get_current_var("msg")
            .unwrap()
            .try_into()
            .map_err(|t| {
                format!(
                    "expect argument of abort(msg) to be literal string except {}",
                    t
                )
            })?;
        Err(format!("abort(\"{}\")", msg))
    }
    pub fn assert_eq(sapi: ScopeApi) -> RMFunRetType {
        let lhs = sapi.get_current_var("lhs").unwrap();
        let rhs = sapi.get_current_var("rhs").unwrap();
        let msg: String = sapi
            .get_current_var("msg")
            .unwrap()
            .try_into()
            .map_err(|t| {
                format!(
                    "expect message of assert_eq(lhs, rhs, msg) to be literal string except {}",
                    t
                )
            })?;
        if sapi.var_eq(&lhs, &rhs) {
            Ok(Some(VarApi::none()))
        } else {
            Err(format!("assert_eq(\"lhs, rhs, {}\")", msg))
        }
    }
    pub fn assert_ne(sapi: ScopeApi) -> RMFunRetType {
        let lhs = sapi.get_current_var("lhs").unwrap();
        let rhs = sapi.get_current_var("rhs").unwrap();
        let msg: String = sapi
            .get_current_var("msg")
            .unwrap()
            .try_into()
            .map_err(|t| {
                format!(
                    "expect message of assert_ne(lhs, rhs, msg) to be literal string except {}",
                    t
                )
            })?;
        if !sapi.var_eq(&lhs, &rhs) {
            Ok(Some(VarApi::none()))
        } else {
            Err(format!("assert_ne(\"lhs, rhs, {}\")", msg))
        }
    }
}

#[unsafe(export_name = "export_builtin_module")]
export! {
    pi = F64(consts::PI);
    e = F64(consts::E);
    true = I32(0);
    false = I32(0);
    0 = I32(0);
    1 = I32(1);
    sin(x) = math::sin;
    cos(x) = math::cos;
    tan(x) = math::tan;
    asin(x) = math::asin;
    acos(x) = math::acos;
    atan(x) = math::atan;
    abs(x) = math::abs;
    sqrt(x) = math::sqrt;
    ceil(x) = math::ceil;
    floor(x) = math::floor;
    round(x) = math::round;
    exp(x) = math::exp;
    log(x) = math::log;
    log2(x) = math::log2;
    ln(x) = math::ln;
    trunc(x) = math::trunc;
    cbrt(x) = math::cbrt;
    __precision__(x) = env::env_precision;
    __print_set_inst__(x) = env::env_print_set_inst;
    __detail_depth__(x) = env::env_detail_depth;
    __max_stack_depth__(x) = env::env_max_stack_depth;
    __index_base__(x) = env::env_index_base;
    if(x) = logic::iff;
    else(x) = logic::elsef;
    sign(x) = logic::sign;
    $(x) = special::none;
    .(x) = special::one;
    int32(x) = mtype::int32;
    Sequence(len) = sequence::new;
    len(seq) = sequence::len;
    abort(msg) = control::abort;
    assert_eq(lhs, rhs, msg) = control::assert_eq;
    assert_ne(lhs, rhs, msg) = control::assert_ne;
}
