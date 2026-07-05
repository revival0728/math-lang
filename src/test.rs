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
