use crate::export;
use crate::rmapi::*;

pub fn sin(sapi: ScopeApi) -> RMApiResult<Option<VarApi>> {
    let x: f64 = sapi
        .get_current_var("x")
        .unwrap()
        .try_into()
        .map_err(|t| format!("expect argument of sin(x) to be a number but got {}", t))?;
    Ok(Some(VarApi::from(x.sin())))
}

export! {
    sin(x) = sin;
}
