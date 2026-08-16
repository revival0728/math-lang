#[cfg(test)]
pub mod examples {
    use std::fs;
    use std::path::Path;

    fn get(path_str: &'static str) -> String {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("examples").join(path_str);
        fs::read_to_string(path).expect("example not found")
    }
    pub fn basic() -> String {
        get("basic.mls")
    }
    pub fn cosine_law() -> String {
        get("cosine-law.mls")
    }
    pub fn fib() -> String {
        get("fib.mls")
    }
    pub fn sequence() -> String {
        get("sequence.mls")
    }
    pub fn module() -> String {
        get("module.mls")
    }
    pub fn rust_module() -> String {
        get("rust-module.mls")
    }
    pub fn cur_operator() -> String {
        get("cur-operator.mls")
    }
}

#[cfg(test)]
pub mod simple_expr {
    pub fn at() -> &'static str {
        "@a = 1"
    }
    pub fn neg_idx() -> &'static str {
        "-a:1^10"
    }
    pub fn neg_expr_1() -> &'static str {
        "-a^10"
    }
    pub fn neg_expr_2() -> &'static str {
        "-(a^10)"
    }
    pub fn expr_1() -> &'static str {
        "i = i + 1"
    }
    pub fn expr_2() -> &'static str {
        "i = i + 1 * b - 1"
    }
    pub fn expr_3() -> &'static str {
        "i = i * (1 + 2) * 3 / b^5 + 4^(a + 3)"
    }
}
