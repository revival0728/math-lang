use super::uint::BigUint;
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone)]
/// Structure stores big numbers in form of `(+/-) cff * 2^(-exp)`
pub struct BigNum {
    sgn: u8, // 0: positive, 1: negative
    cff: BigUint,
    exp: u32,
}

impl BigNum {
    pub fn new() -> Self {
        Self {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
        }
    }
}

impl Debug for BigNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})({})*(2^(-{}))",
            if self.sgn == 0 { "+" } else { "-" },
            self.cff,
            self.exp
        )
    }
}

impl BigNum {
    pub fn align_exp(a: &mut Self, b: &mut Self) {
        if a.exp == b.exp {
            return;
        }
        let target = std::cmp::max(a.exp, b.exp);
        a.cff.bits <<= target - a.exp;
        b.cff.bits <<= target - b.exp;
        a.exp = target;
        b.exp = target;
    }
}

impl AddAssign<&Self> for BigNum {
    fn add_assign(&mut self, rhs: &Self) {
        let mut rhs = rhs.clone();
        Self::align_exp(self, &mut rhs);
        if self.sgn ^ rhs.sgn == 0 {
            self.cff += &rhs.cff;
        } else {
            if self.cff < rhs.cff {
                std::mem::swap(self, &mut rhs);
            }
            self.cff -= &rhs.cff;
        }
    }
}

impl Add<&Self> for BigNum {
    type Output = BigNum;
    fn add(mut self, rhs: &Self) -> Self::Output {
        self += &rhs;
        self
    }
}

impl Add for &BigNum {
    type Output = BigNum;
    fn add(self, rhs: Self) -> Self::Output {
        self.clone() + rhs
    }
}
