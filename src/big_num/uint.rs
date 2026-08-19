use crate::builtin::mtype;

use super::ubits::UintBits;
use std::cmp::Ord;
use std::convert::From;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

// TODO: div_rem, Div, Rem, div_num((Self, (Self, usize)) -> (integer, (decimal, -2^N)))
// TODO: std::fmt::Display using double dabble and reverse double dabble

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

impl Ord for BigUint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match self.bits.len().cmp(&other.bits.len()) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => {
                for (sb, ob) in self.bits.iter().rev().zip(other.bits.iter().rev()) {
                    match sb.cmp(&ob) {
                        Ordering::Greater => return Ordering::Greater,
                        Ordering::Less => return Ordering::Less,
                        Ordering::Equal => continue,
                    };
                }
                Ordering::Equal
            }
        }
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

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

impl MulAssign<&Self> for BigUint {
    fn mul_assign(&mut self, rhs: &Self) {
        let mut res = BigUint::new();
        for bit in rhs.bits.iter() {
            if bit == 1 {
                res += self;
            }
            self.bits <<= 1;
        }
        *self = res;
    }
}

impl Mul<Self> for &BigUint {
    type Output = BigUint;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        ret *= rhs;
        ret
    }
}

impl Mul<&Self> for BigUint {
    type Output = BigUint;
    fn mul(mut self, rhs: &Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl BigUint {
    /// return quotient while self is remainder
    pub fn div_rem_self(&mut self, rhs: &Self) -> Self {
        let sbl = self.bits.bit_len();
        let rbl = rhs.bits.bit_len();
        if sbl < rbl {
            return Self::new();
        }
        let mut rhs = rhs.clone();
        let qlen = sbl - rbl;
        for _ in 0..qlen {
            rhs.bits <<= 1;
        }
        let mut q = Self::new();
        for i in 0..=qlen {
            if *self >= rhs {
                *self -= &rhs;
                q.bits.set(0);
            }
            if i < qlen {
                q.bits <<= 1;
            }
            rhs.bits >>= 1;
        }
        q
    }
    /// return (quotient, remainder)
    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        let mut r = self.clone();
        let q = r.div_rem_self(rhs);
        (q, r)
    }
}

impl DivAssign<&Self> for BigUint {
    fn div_assign(&mut self, rhs: &Self) {
        *self = self.div_rem_self(rhs);
    }
}

impl Div<Self> for &BigUint {
    type Output = BigUint;
    fn div(self, rhs: Self) -> Self::Output {
        self.clone().div_rem_self(rhs)
    }
}

impl Div<&Self> for BigUint {
    type Output = BigUint;
    fn div(mut self, rhs: &Self) -> Self::Output {
        self.div_rem_self(rhs)
    }
}

impl RemAssign<&Self> for BigUint {
    fn rem_assign(&mut self, rhs: &Self) {
        self.div_rem_self(rhs);
    }
}

impl Rem<Self> for &BigUint {
    type Output = BigUint;
    fn rem(self, rhs: Self) -> Self::Output {
        let mut r = self.clone();
        r.div_rem_self(rhs);
        r
    }
}

impl Rem<&Self> for BigUint {
    type Output = BigUint;
    fn rem(mut self, rhs: &Self) -> Self::Output {
        self.div_rem_self(rhs);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ord_and_partial_ord() {
        let a = BigUint::from(1_u32);
        let b = BigUint::from(3_u32);
        let c = BigUint::from(UintBits::from(vec![1, 1]));
        assert_eq!(a > b, false);
        assert_eq!(a >= b, false);
        assert_eq!(a < b, true);
        assert_eq!(a <= b, true);
        assert_eq!(a == b, false);
        assert_eq!(a == a, true);
        assert_eq!(b == b, true);
        assert_eq!(c == c, true);
        assert_eq!(a > c, false);
        assert_eq!(a >= c, false);
        assert_eq!(a < c, true);
        assert_eq!(a <= c, true);
    }

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

    #[test]
    fn mul_assign() {
        let mut a = BigUint::from(2_u32);
        let b = BigUint::from(3_u32);
        a *= &b;
        assert_eq!(a, BigUint::from(6_u32));
    }

    #[test]
    fn mul() {
        let a = BigUint::from(2_u32);
        let b = BigUint::from(3_u32);
        let c = BigUint::from(1_u32);
        let large = BigUint::from(UintBits::from(vec![1 | (1_u64 << 63), 1]));
        assert_eq!(&a * &b, BigUint::from(6_u32));
        assert_eq!(&a * &b * &c, BigUint::from(6_u32));
        assert_eq!(
            &b * &large,
            BigUint::from(UintBits::from(vec![3 | (1_u64 << 63), 4]))
        );
    }

    #[test]
    fn div_rem() {
        let a = BigUint::from(7_u32);
        let b = BigUint::from(3_u32);
        let large = BigUint::from(UintBits::from(vec![1, 1]));
        assert_eq!(a.div_rem(&b), (BigUint::from(2_u32), BigUint::from(1_u32)));
        assert_eq!(
            large.div_rem(&b),
            (BigUint::from(6148914691236517205_u64), BigUint::from(2_u32))
        );
    }

    #[test]
    fn div_assign() {
        let mut a = BigUint::from(17_u32);
        let b = BigUint::from(4_u32);
        let c = BigUint::from(3_u32);
        a /= &b;
        a /= &c;
        assert_eq!(a, BigUint::from(1_u32));
    }

    #[test]
    fn div() {
        let a = BigUint::from(17_u32);
        let b = BigUint::from(4_u32);
        let c = BigUint::from(3_u32);
        assert_eq!(&a / &b / &c, BigUint::from(1_u32));
    }

    #[test]
    fn rem_assign() {
        let mut a = BigUint::from(17_u32);
        let b = BigUint::from(6_u32);
        let c = BigUint::from(3_u32);
        a %= &b;
        a %= &c;
        assert_eq!(a, BigUint::from(2_u32));
    }

    #[test]
    fn rem() {
        let a = BigUint::from(17_u32);
        let b = BigUint::from(6_u32);
        let c = BigUint::from(3_u32);
        assert_eq!(&a % &b % &c, BigUint::from(2_u32));
    }
}
