use super::uint::BigUint;
use std::convert::From;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

const IRR_COND: usize = (u64::BITS * 10) as usize;

// FIXME: impl better eq
#[derive(Debug, Clone, PartialEq, Eq)]
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
            num: BigUint::new(),
            den: BigUint::from(1_u32),
            irr: true,
        }
    }
}

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
        write!(f, "({})/({})", self.num, self.den)
    }
}

impl BigNum {
    pub fn to_float_str(&self, precision: u8) -> String {
        let (q, r) = self.num.div_rem(&self.den);
        let d = r.div_decimal(&self.den, precision);
        format!("{}.{}", q, d)
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
