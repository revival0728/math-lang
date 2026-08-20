use super::uint::BigUint;

pub struct BigNum {
    num: BigUint,
    den: BigUint,
    irr: bool,
}

impl BigNum {
    pub fn new() -> Self {
        Self {
            num: BigUint::new(),
            den: BigUint::from(1_u32),
            irr: true,
        }
    }
}
