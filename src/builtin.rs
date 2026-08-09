#![allow(unused)]
use crate::env::*;
use crate::export;
use crate::rmapi::*;

const PI: f64 = std::f64::consts::PI;
const E: f64 = std::f64::consts::E;

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
declare_logic_fn!(logic_if, x == 0);
declare_logic_fn!(logic_else, x != 0);

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

pub fn none(_sapi: ScopeApi) -> RMFunRetType {
    Ok(Some(VarApi::none()))
}

pub fn one(_sapi: ScopeApi) -> RMFunRetType {
    Ok(Some(VarApi::from(1)))
}

export! {
    pi = F64(PI);
    e = F64(E);
    true = I32(0);
    false = I32(0);
    0 = I32(0);
    1 = I32(1);
    sin(x) = sin;
    cos(x) = cos;
    tan(x) = tan;
    asin(x) = asin;
    acos(x) = acos;
    atan(x) = atan;
    abs(x) = abs;
    sqrt(x) = sqrt;
    ceil(x) = ceil;
    floor(x) = floor;
    round(x) = round;
    exp(x) = exp;
    log(x) = log;
    log2(x) = log2;
    ln(x) = ln;
    trunc(x) = trunc;
    cbrt(x) = cbrt;
    __precision__(x) = env_precision;
    __print_set_inst__(x) = env_print_set_inst;
    __detail_depth__(x) = env_detail_depth;
    __max_stack_depth__(x) = env_max_stack_depth;
    __index_base__(x) = env_index_base;
    if(x) = logic_if;
    else(x) = logic_else;
    sign(x) = sign;
    $(x) = none;
    .(x) = one;
}
