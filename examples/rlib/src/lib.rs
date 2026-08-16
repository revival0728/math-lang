use math_lang::prelude::*;

pub fn rust_add(left: i64, right: i64) -> i64 {
    left + right
}

pub fn add(sapi: ScopeApi) -> RMFunRetType {
    let a: i64 = sapi
        .get_current_var("a")
        .unwrap()
        .try_into()
        .map_err(|t| format!("expected I32 type got {} type", t))?;
    let b: i64 = sapi
        .get_current_var("b")
        .unwrap()
        .try_into()
        .map_err(|t| format!("expected I32 type got {} type", t))?;
    Ok(Some(VarApi::from(rust_add(a, b))))
}

export! {
    LUCKY = I32(0923);
    add_i64(a, b) = add;
}

#[cfg(test)]
mod tests {
    #[test]
    fn rust_add() {
        let result = super::rust_add(2, 2);
        assert_eq!(result, 4);
    }
}
