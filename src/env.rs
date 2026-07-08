// usage of static mut is SAFE: no multiple threads
pub static mut PRECISION: u32 = 7;
pub static mut DETAIL_DEPTH: u32 = 0;

pub fn is_env<'f>(name: &'f str) -> bool {
    name.starts_with("__") && name.ends_with("__")
}
