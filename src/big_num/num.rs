#![allow(unused)]

use super::uint::BigUint;
use std::convert::From;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

const IRR_COND: usize = (u64::BITS * 10) as usize;

#[derive(Debug, Clone)]
pub struct BigNum {
    sgn: u8, // 0: positive, 1: negative
    num: BigUint,
    den: BigUint,
    irr: bool,
}

impl BigNum {
    pub fn new() -> Self {
        Self {
            sgn: 0,
            num: BigUint::from(0_u32),
            den: BigUint::from(1_u32),
            irr: true,
        }
    }
    pub fn inf() -> Self {
        Self {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(0_u32),
            irr: true,
        }
    }
}

macro_rules! impl_from_signed_integer {
    ($type:ty, $utype:ty) => {
        impl From<$type> for BigNum {
            fn from(value: $type) -> Self {
                if value >= 0 {
                    Self {
                        sgn: 0,
                        num: BigUint::from(value as $utype),
                        den: BigUint::from(1_u32),
                        irr: true,
                    }
                } else {
                    Self {
                        sgn: 1,
                        num: BigUint::from((-value) as $utype),
                        den: BigUint::from(1_u32),
                        irr: true,
                    }
                }
            }
        }
    };
}
impl_from_signed_integer!(i8, u8);
impl_from_signed_integer!(i32, u32);
impl_from_signed_integer!(i64, u64);
impl_from_signed_integer!(i128, u128);

macro_rules! impl_from_unsigned_integer {
    ($type:ty) => {
        impl From<$type> for BigNum {
            fn from(value: $type) -> Self {
                Self {
                    sgn: 0,
                    num: BigUint::from(value),
                    den: BigUint::from(1_u32),
                    irr: true,
                }
            }
        }
    };
}
impl_from_unsigned_integer!(u8);
impl_from_unsigned_integer!(u32);
impl_from_unsigned_integer!(u64);
impl_from_unsigned_integer!(u128);

macro_rules! impl_from_float {
    ($type:ty, $precision:literal) => {
        impl From<$type> for BigNum {
            /// only support value ranges in `[-2^128, 2^128]`
            fn from(value: $type) -> Self {
                let sgn = value.is_sign_negative() as u8;
                if value.is_infinite() {
                    return Self {
                        sgn,
                        num: BigUint::from(1_u32),
                        den: BigUint::from(0_u32),
                        irr: true,
                    };
                }
                const PRECISOIN: u32 = $precision;
                let int = unsafe { value.abs().to_int_unchecked::<u128>() };
                let fra = unsafe {
                    (value.fract() * (10.0 as $type).powi(PRECISOIN as i32))
                        .to_int_unchecked::<u64>()
                };
                let mut res = Self::from(int);
                res += &BigNum {
                    sgn: 0,
                    num: BigUint::from(fra),
                    den: BigUint::from(10_u64.pow(PRECISOIN)),
                    irr: false,
                };
                res
            }
        }
    };
}
impl_from_float!(f32, 7);
impl_from_float!(f64, 15);

fn uint_gcd(a: &BigUint, b: &BigUint) -> BigUint {
    if b.is_zero() {
        a.clone()
    } else {
        uint_gcd(b, &(a % b))
    }
}

impl BigNum {
    fn common_both(a: &mut Self, b: &mut Self) {
        if a.den == b.den {
            return;
        }
        a.num *= &b.den;
        b.num *= &a.den;
        a.den *= &b.den;
        b.den = a.den.clone();
        a.irr = false;
        b.irr = false;
    }
    // fn common(&mut self, other: &Self) {
    //     if self.den == other.den {
    //         return;
    //     }
    //     self.num *= &other.den;
    //     self.den *= &other.den;
    //     self.irr = false;
    // }
    fn reduce(&mut self) {
        if self.irr {
            return;
        }
        if self.den.is_zero() {
            self.num = BigUint::from(1_u32);
            self.irr = true;
            return;
        }
        let gcd = uint_gcd(&self.num, &self.den);
        self.num /= &gcd;
        self.den /= &gcd;
        self.irr = true;
    }
    /// Update `self.irr` after operation
    fn update_irr(&mut self) {
        self.irr = false;
        if self.num.bit_count() >= IRR_COND || self.den.bit_count() >= IRR_COND {
            self.reduce();
        }
    }
}

impl PartialEq for BigNum {
    fn eq(&self, other: &Self) -> bool {
        let mut s = self.clone();
        let mut o = other.clone();
        s.reduce();
        o.reduce();
        s.sgn == o.sgn && s.num == o.num && s.den == o.den
    }
}

impl Eq for BigNum {}

impl BigNum {
    /// PartialEq without reduction first
    fn eq_raw(&self, other: &Self) -> bool {
        self.sgn == other.sgn && self.num == other.num && self.den == other.den
    }
}

impl Neg for &BigNum {
    type Output = BigNum;
    fn neg(self) -> Self::Output {
        let mut ret = self.clone();
        ret.sgn ^= 1;
        ret
    }
}

impl Neg for BigNum {
    type Output = BigNum;
    fn neg(mut self) -> Self::Output {
        self.sgn ^= 1;
        self
    }
}

impl AddAssign<&Self> for BigNum {
    fn add_assign(&mut self, rhs: &Self) {
        let mut rhs = rhs.clone();
        Self::common_both(self, &mut rhs);
        if self.sgn ^ rhs.sgn == 0 {
            self.num += &rhs.num;
        } else {
            if self.num >= rhs.num {
                self.num -= &rhs.num;
            } else {
                self.sgn ^= 1;
                self.num = &rhs.num - &self.num;
            }
        }
        self.update_irr();
    }
}

impl Add for &BigNum {
    type Output = BigNum;
    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        ret += rhs;
        ret
    }
}

impl Add<&Self> for BigNum {
    type Output = BigNum;
    fn add(mut self, rhs: &Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign<&Self> for BigNum {
    fn sub_assign(&mut self, rhs: &Self) {
        let mut rhs = rhs.clone();
        Self::common_both(self, &mut rhs);
        if self.sgn ^ rhs.sgn == 1 {
            self.num += &rhs.num;
        } else {
            if self.num >= rhs.num {
                self.num -= &rhs.num;
            } else {
                self.sgn ^= 1;
                self.num = &rhs.num - &self.num;
            }
        }
        self.update_irr();
    }
}

impl Sub for &BigNum {
    type Output = BigNum;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        ret -= rhs;
        ret
    }
}

impl Sub<&Self> for BigNum {
    type Output = BigNum;
    fn sub(mut self, rhs: &Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl MulAssign<&Self> for BigNum {
    fn mul_assign(&mut self, rhs: &Self) {
        self.sgn ^= rhs.sgn;
        self.num *= &rhs.num;
        self.den *= &rhs.den;
        self.update_irr();
    }
}

impl Mul for &BigNum {
    type Output = BigNum;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        ret *= rhs;
        ret
    }
}

impl Mul<&Self> for BigNum {
    type Output = BigNum;
    fn mul(mut self, rhs: &Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl DivAssign<&Self> for BigNum {
    fn div_assign(&mut self, rhs: &Self) {
        self.sgn ^= rhs.sgn;
        self.num *= &rhs.den;
        self.den *= &rhs.num;
        self.update_irr();
    }
}

impl Div for &BigNum {
    type Output = BigNum;
    fn div(self, rhs: Self) -> Self::Output {
        let mut ret = self.clone();
        ret /= rhs;
        ret
    }
}

impl Div<&Self> for BigNum {
    type Output = BigNum;
    fn div(mut self, rhs: &Self) -> Self::Output {
        self /= rhs;
        self
    }
}

impl Display for BigNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.den.is_zero() {
            write!(f, "{}{}", if self.sgn == 0 { "+" } else { "-" }, "INF")
        } else {
            write!(f, "({})/({})", self.num, self.den)
        }
    }
}

impl BigNum {
    pub fn to_float_str(&self, precision: u8) -> String {
        if self.den.is_zero() {
            return format!("{}{}", if self.sgn == 0 { "+" } else { "-" }, "INF");
        }
        let (q, r) = self.num.div_rem(&self.den);
        let d = r.div_decimal(&self.den, precision);
        format!("{}.{}", q, d)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eq_and_eq_raw() {
        let a = BigNum::from(1_u32);
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(2_u32),
            den: BigUint::from(2_u32),
            irr: false,
        };
        assert_eq!(a == b, true);
        assert_eq!(a.eq_raw(&b), false);
    }

    #[test]
    fn inf() {
        let inf = BigNum::inf();
        let num = BigNum::from(5_u32);
        assert_eq!((&inf + &num).to_string(), "+INF");
        assert_eq!((&inf - &num).to_string(), "+INF");
        assert_eq!((&inf * &num).to_string(), "+INF");
        assert_eq!((&inf / &num).to_string(), "+INF");
        assert_eq!((&num - &inf).to_string(), "-INF");
        assert_eq!((&(-(&inf)) + &num).to_string(), "-INF");
        assert_eq!((&(-(&inf)) - &num).to_string(), "-INF");
        assert_eq!((&(-(&inf)) * &num).to_string(), "-INF");
        assert_eq!((&(-(&inf)) / &num).to_string(), "-INF");
    }

    #[test]
    fn from_integer() {
        let ineg = BigNum::from(-1_i32);
        let ipos = BigNum::from(1_i32);
        let u = BigNum::from(1_i32);
        assert_eq!(
            ineg,
            BigNum {
                sgn: 1,
                num: BigUint::from(1_u32),
                den: BigUint::from(1_u32),
                irr: true,
            }
        );
        assert_eq!(
            ipos,
            BigNum {
                sgn: 0,
                num: BigUint::from(1_u32),
                den: BigUint::from(1_u32),
                irr: true,
            }
        );
        assert_eq!(
            u,
            BigNum {
                sgn: 0,
                num: BigUint::from(1_u32),
                den: BigUint::from(1_u32),
                irr: true,
            }
        );
    }

    #[test]
    fn from_float_to_string() {
        let half = BigNum::from(1.142857_f32);
        let full = BigNum::from(1.142857142857_f64);
        let pi = BigNum::from(std::f64::consts::PI);
        let e = BigNum::from(std::f64::consts::E);
        assert_eq!(half.to_float_str(6), "1.142856");
        assert_eq!(full.to_float_str(12), "1.142857142856");
        assert_eq!(pi.to_float_str(10), "3.1415926535");
        assert_eq!(e.to_float_str(10), "2.7182818284");
    }

    #[test]
    fn test_uint_gcd() {
        let a = BigUint::from(15_u32);
        let b = BigUint::from(12_u32);
        assert_eq!(uint_gcd(&a, &b), BigUint::from(3_u32));
    }

    #[test]
    fn neg() {
        let a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        assert_eq!(
            -(&a),
            BigNum {
                sgn: 1,
                num: BigUint::from(1_u32),
                den: BigUint::from(2_u32),
                irr: true,
            }
        );
        assert_eq!(
            -a,
            BigNum {
                sgn: 1,
                num: BigUint::from(1_u32),
                den: BigUint::from(2_u32),
                irr: true,
            }
        );
    }

    #[test]
    fn common_reduce() {
        let mut a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let mut b = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        BigNum::common_both(&mut a, &mut b);
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                num: BigUint::from(3_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
        assert_eq!(
            b,
            BigNum {
                sgn: 0,
                num: BigUint::from(2_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
        a.reduce();
        b.reduce();
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                num: BigUint::from(1_u32),
                den: BigUint::from(2_u32),
                irr: true,
            }
        );
        assert_eq!(
            b,
            BigNum {
                sgn: 0,
                num: BigUint::from(1_u32),
                den: BigUint::from(3_u32),
                irr: true,
            }
        );
    }

    #[test]
    fn add_assign() {
        let mut a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        a += &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                num: BigUint::from(5_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
    }

    #[test]
    fn add() {
        let a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        let c = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(6_u32),
            irr: true,
        };
        let d = BigNum {
            sgn: 1,
            num: BigUint::from(3_u32),
            den: BigUint::from(1_u32),
            irr: true,
        };
        assert_eq!(
            &a + &b,
            BigNum {
                sgn: 0,
                num: BigUint::from(5_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
        assert_eq!(
            &a + &b + &c,
            BigNum {
                sgn: 0,
                num: BigUint::from(6_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
        assert_eq!(
            &a + &b + &c + &d,
            BigNum {
                sgn: 1,
                num: BigUint::from(12_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
    }

    #[test]
    fn sub_assign() {
        let mut a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        a -= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                num: BigUint::from(1_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
    }

    #[test]
    fn sub() {
        let a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        let c = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(6_u32),
            irr: true,
        };
        let d = BigNum {
            sgn: 1,
            num: BigUint::from(3_u32),
            den: BigUint::from(1_u32),
            irr: true,
        };
        assert_eq!(
            &a - &b,
            BigNum {
                sgn: 0,
                num: BigUint::from(1_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
        assert_eq!(
            &a - &b - &c,
            BigNum {
                sgn: 0,
                num: BigUint::from(0_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
        assert_eq!(
            &a - &b - &c - &d,
            BigNum {
                sgn: 0,
                num: BigUint::from(18_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
        assert_eq!(
            &a - &b - &c - &(-(&d)),
            BigNum {
                sgn: 1,
                num: BigUint::from(18_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
    }

    #[test]
    fn mul_assign() {
        let mut a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(2_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        a *= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                num: BigUint::from(2_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
    }

    #[test]
    fn mul() {
        let a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(2_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        let c = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(6_u32),
            irr: true,
        };
        let d = BigNum {
            sgn: 1,
            num: BigUint::from(3_u32),
            den: BigUint::from(1_u32),
            irr: true,
        };
        assert_eq!(
            &a * &b,
            BigNum {
                sgn: 0,
                num: BigUint::from(2_u32),
                den: BigUint::from(6_u32),
                irr: false,
            }
        );
        assert_eq!(
            &a * &b * &c,
            BigNum {
                sgn: 0,
                num: BigUint::from(2_u32),
                den: BigUint::from(36_u32),
                irr: false,
            }
        );
        assert_eq!(
            &a * &b * &c * &d,
            BigNum {
                sgn: 1,
                num: BigUint::from(6_u32),
                den: BigUint::from(36_u32),
                irr: false,
            }
        );
    }

    #[test]
    fn div_assign() {
        let mut a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(2_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        a /= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                num: BigUint::from(3_u32),
                den: BigUint::from(4_u32),
                irr: false,
            }
        );
    }

    #[test]
    fn div() {
        let a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(2_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        let c = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(6_u32),
            irr: true,
        };
        let d = BigNum {
            sgn: 1,
            num: BigUint::from(3_u32),
            den: BigUint::from(1_u32),
            irr: true,
        };
        assert_eq!(
            &a / &b,
            BigNum {
                sgn: 0,
                num: BigUint::from(3_u32),
                den: BigUint::from(4_u32),
                irr: false,
            }
        );
        assert_eq!(
            &a / &b / &c / &d,
            BigNum {
                sgn: 1,
                num: BigUint::from(18_u32),
                den: BigUint::from(12_u32),
                irr: false,
            }
        );
    }

    #[test]
    fn to_float_str() {
        let a = BigNum {
            sgn: 0,
            num: BigUint::from(1_u32),
            den: BigUint::from(2_u32),
            irr: true,
        };
        let b = BigNum {
            sgn: 0,
            num: BigUint::from(2_u32),
            den: BigUint::from(3_u32),
            irr: true,
        };
        let c = BigNum {
            sgn: 0,
            num: BigUint::from(8_u32),
            den: BigUint::from(7_u32),
            irr: true,
        };
        assert_eq!(a.to_float_str(6), "0.500000");
        assert_eq!(b.to_float_str(6), "0.666666");
        assert_eq!(c.to_float_str(6), "1.142857");
    }
}
