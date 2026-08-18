use super::ubits::UintBits;
use std::convert::From;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BigUint {
    bits: UintBits,
}

impl BigUint {
    pub fn new() -> Self {
        Self {
            bits: UintBits::new(),
        }
    }
}

impl From<UintBits> for BigUint {
    fn from(value: UintBits) -> Self {
        Self { bits: value }
    }
}

macro_rules! impl_from {
    ($from_type:ty) => {
        impl From<$from_type> for BigUint {
            fn from(value: $from_type) -> Self {
                let mut bits = UintBits::new();
                bits.set_bits(0, value as u64);
                Self { bits }
            }
        }
    };
}
impl_from!(u8);
impl_from!(u32);
impl_from!(u64);

impl AddAssign<&Self> for BigUint {
    fn add_assign(&mut self, rhs: &Self) {
        let mut carry = rhs.bits.clone();
        while !carry.all_zero() {
            let nxt_carry = ((&self.bits) & (&carry)) << 1;
            self.bits ^= &carry;
            carry = nxt_carry;
        }
    }
}

impl Add<Self> for &BigUint {
    type Output = BigUint;
    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        ret += rhs;
        ret
    }
}

impl Add<&Self> for BigUint {
    type Output = BigUint;
    fn add(mut self, rhs: &Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign<&Self> for BigUint {
    fn sub_assign(&mut self, rhs: &Self) {
        let mut carry = rhs.bits.clone();
        let max_len = std::cmp::max(carry.len(), self.bits.len()) << 1;
        while !carry.all_zero() && carry.len() <= max_len {
            let nxt_carry = (((&self.bits) ^ (&carry)) & (&carry)) << 1;
            self.bits ^= &carry;
            carry = nxt_carry;
        }
        self.bits.truncate(max_len >> 1);
    }
}

impl Sub<Self> for &BigUint {
    type Output = BigUint;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        ret -= rhs;
        ret
    }
}

impl Sub<&Self> for BigUint {
    type Output = BigUint;
    fn sub(mut self, rhs: &Self) -> Self::Output {
        self -= rhs;
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_assign() {
        let mut lhs = BigUint::from(1_u32);
        let rhs = BigUint::from(2_u32);
        lhs += &rhs;
        assert_eq!(lhs, BigUint::from(3_u32));
    }

    #[test]
    fn add() {
        let a = BigUint::from(1_u32);
        let b = BigUint::from(2_u32);
        let c = BigUint::from(3_u32);
        let not_a = BigUint::from(u64::MAX - 1);
        let large1 = BigUint::from(UintBits::from(vec![1, 1]));
        let large2 = BigUint::from(UintBits::from(vec![u64::MAX]));
        assert_eq!(&a + &b, BigUint::from(3_u32));
        assert_eq!(&a + &b + &c, BigUint::from(6_u32));
        assert_eq!(&a + &large1, BigUint::from(UintBits::from(vec![2, 1])));
        assert_eq!(
            &not_a + &large1 + &a,
            BigUint::from(UintBits::from(vec![0, 2]))
        );
        assert_eq!(&large2 + &a, BigUint::from(UintBits::from(vec![0, 1])));
    }

    #[test]
    fn sub_assign() {
        let mut lhs = BigUint::from(2_u32);
        let rhs = BigUint::from(1_u32);
        lhs -= &rhs;
        assert_eq!(lhs, BigUint::from(1_u32));
    }

    #[test]
    fn sub() {
        let a = BigUint::from(1_u32);
        let b = BigUint::from(2_u32);
        let c = BigUint::from(3_u32);
        let large = BigUint::from(UintBits::from(vec![1, 1]));
        assert_eq!(&b - &a, BigUint::from(1_u32));
        assert_eq!(&c - &b - &a, BigUint::from(0_u32));
        assert_eq!(&a - &b, BigUint::from(u64::MAX));
        assert_eq!(&large - &a, BigUint::from(UintBits::from(vec![0, 1])));
        assert_eq!(
            &a - &large,
            BigUint::from(UintBits::from(vec![0, u64::MAX]))
        );
    }
}
