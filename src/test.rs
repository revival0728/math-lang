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
}

#[cfg(test)]
pub mod simple_expr {
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
