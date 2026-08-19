use super::ubits::UintBits;

#[derive(Debug, Clone)]
pub struct Decimal {
    bits: UintBits,
    precision: u8,
}

impl Decimal {
    pub fn new(bits: UintBits, precision: u8) -> Self {
        Self { bits, precision }
    }
}
