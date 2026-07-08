pub static mut PRECISION: u32 = 7;

pub fn is_env<'f>(name: &'f str) -> bool {
    name.starts_with("__") && name.ends_with("__")
}
