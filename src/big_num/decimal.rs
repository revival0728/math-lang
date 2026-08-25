use super::ubits::UintBits;
use super::uint::BigUint;
use std::fmt::Display;

/// Structure stores decimal part in big endian bits
///
/// `base` stands for total actual bits in this structure,
/// preventing leading zero confusion
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decimal {
    pub bits: UintBits,
    pub base: u32, // maximum of base is 848 (from 10-base precision 255)
    pub prec: u8,
}

impl Decimal {
    pub fn new(bits: UintBits, base: u32, prec: u8) -> Self {
        Self { bits, base, prec }
    }
}

impl Display for Decimal {
    // TODO: Fix leading 0 error
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const TLOG2: f32 = 0.30103;
        let mut deci = BigUint::from(self.bits.clone());
        let base2 = {
            let mut bits = UintBits::new();
            bits.set(self.base as usize);
            BigUint::from(bits)
        };
        let mut exp = (self.base as f32 * TLOG2).trunc() as u32;
        let mut base10 = BigUint::from(10_u32);
        while exp > 0 {
            if exp & 1 == 1 {
                deci *= &base10;
            }
            base10 *= &base10.clone();
            exp >>= 1;
        }
        deci /= &base2;
        let mut res = deci.to_string();
        let prec_usize = self.prec as usize;
        if res.len() > prec_usize {
            res.truncate(prec_usize);
        } else {
            res.push_str(&String::from_utf8(vec!['0' as u8; prec_usize - res.len()]).unwrap());
        }
        write!(f, "{}", res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_to_string() {
        let one = BigUint::from(1_u32);
        let a = BigUint::from(2_u32);
        let b = BigUint::from(3_u32);
        let c = BigUint::from(7_u32);
        let d = BigUint::from(700_u32);
        assert_eq!(one.div_decimal(&a, 6).to_string(), "500000");
        assert_eq!(one.div_decimal(&a, 15).to_string(), "500000000000000");
        assert_eq!(one.div_decimal(&b, 6).to_string(), "333333");
        assert_eq!(one.div_decimal(&b, 15).to_string(), "333333333333333");
        assert_eq!(one.div_decimal(&c, 6).to_string(), "142857");
        assert_eq!(one.div_decimal(&c, 15).to_string(), "142857142857142");
        assert_eq!(one.div_decimal(&d, 15).to_string(), "001428571428571");
        assert_eq!(one.div_decimal(&c, 255).to_string().len(), 255);
    }
}
