use super::ubits::UintBits;

/// Structure stores decimal part in big endian bits
///
/// `base` stands for total actual bits in this structure,
/// preventing leading zero confusion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decimal {
    bits: UintBits,
    base: u32,
}

impl Decimal {
    pub fn new(bits: UintBits, base: u32) -> Self {
        Self { bits, base }
    }
}
