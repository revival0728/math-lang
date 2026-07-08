// usage of static mut is SAFE: no multiple threads
pub static mut PRECISION: u32 = 7;
pub static mut PRINT_SET_INST: u32 = 1;
pub static mut DETAIL_DEPTH: u32 = 0;
pub static mut MAX_STACK_DEPTH: u32 = 512;

pub fn is_env<'f>(name: &'f str) -> bool {
    name.starts_with("__") && name.ends_with("__")
}
