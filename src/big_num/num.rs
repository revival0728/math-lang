use super::uint::BigUint;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use std::convert::{From, TryFrom};
use std::fmt::{Debug, Display};
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

macro_rules! impl_from_uint {
    ($type:ty) => {
        impl From<$type> for BigNum {
            fn from(value: $type) -> Self {
                Self {
                    sgn: 0,
                    cff: BigUint::from(value),
                    exp: 0,
                }
            }
        }
    };
}
impl_from_uint!(u8);
impl_from_uint!(u32);
impl_from_uint!(u64);
impl_from_uint!(u128);

macro_rules! impl_from_int {
    ($type:ty, $utype:ty) => {
        impl From<$type> for BigNum {
            fn from(value: $type) -> Self {
                let sgn = if value < 0 { 1 } else { 0 };
                Self {
                    sgn,
                    cff: BigUint::from(value.abs() as $utype),
                    exp: 0,
                }
            }
        }
    };
}
impl_from_int!(i8, u8);
impl_from_int!(i32, u32);
impl_from_int!(i64, u64);
impl_from_int!(i128, u128);

impl From<BigUint> for BigNum {
    fn from(value: BigUint) -> Self {
        Self {
            sgn: 0,
            cff: value,
            exp: 0,
        }
    }
}

macro_rules! impl_from_float {
    ($type:ty, $uit:ty, $prec:literal) => {
        impl From<$type> for BigNum {
            fn from(value: $type) -> Self {
                const PRECISION: u8 = $prec;
                let sgn = if value.signum() < 0.0 { 1 } else { 0 };
                let value = value.abs();
                let int = value.trunc() as $uit;
                let dec =
                    (value.fract() * (10 as $type).powi(PRECISION as i32) as $type).trunc() as $uit;
                let p10 = (10 as $uit).pow(PRECISION as u32);
                eprintln!("INT: {}, DEC: {}", int, dec);

                let int = BigNum::from(int);
                let dec = BigNum::from(dec);
                let p10 = BigNum::from(p10);
                let mut ret = int + &(&dec / &p10);
                ret.sgn = sgn;
                ret
            }
        }
    };
}
impl_from_float!(f32, u32, 7);
impl_from_float!(f64, u64, 15);

impl Display for BigNum {
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

impl Debug for BigNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl BigNum {
    pub fn float_to_str(&self, precision: u8) -> String {
        let pow2 = BigUint::pow2(self.exp);
        let (int, dec) = self.cff.div_rem(&pow2);
        let dec = dec.div_decimal(&pow2, precision);
        format!("{}{}.{}", if self.sgn == 1 { "-" } else { "" }, int, dec)
    }
}

impl PartialEq for BigNum {
    fn eq(&self, other: &Self) -> bool {
        if self.cff.is_zero() && other.cff.is_zero() {
            return true;
        }
        if self.sgn != other.sgn {
            return false;
        }
        if self.exp == other.exp {
            return self.cff == other.cff;
        }
        let mut s = self.clone();
        let mut o = other.clone();
        Self::align_exp(&mut s, &mut o);
        s.cff == o.cff
    }
}

impl Eq for BigNum {}

impl Ord for BigNum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::{Equal, Greater, Less};
        if self.cff.is_zero() && other.cff.is_zero() {
            return Equal;
        }
        match self.sgn.cmp(&other.sgn) {
            Less => Greater,
            Greater => Less,
            Equal => {
                if self.exp == other.exp {
                    return self.cff.cmp(&other.cff);
                }
                let mut s = self.clone();
                let mut o = other.clone();
                Self::align_exp(&mut s, &mut o);
                s.cff.cmp(&o.cff)
            }
        }
    }
}

impl PartialOrd for BigNum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
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
                self.sgn ^= 1;
                std::mem::swap(&mut self.cff, &mut rhs.cff);
            }
            self.cff -= &rhs.cff;
        }
    }
}

impl SubAssign<&Self> for BigNum {
    fn sub_assign(&mut self, rhs: &Self) {
        let mut rhs = rhs.clone();
        Self::align_exp(self, &mut rhs);
        if self.sgn ^ rhs.sgn == 1 {
            self.cff += &rhs.cff;
        } else {
            if self.cff < rhs.cff {
                self.sgn ^= 1;
                std::mem::swap(&mut self.cff, &mut rhs.cff);
            }
            self.cff -= &rhs.cff;
        }
    }
}

impl MulAssign<&Self> for BigNum {
    fn mul_assign(&mut self, rhs: &Self) {
        self.sgn ^= rhs.sgn;
        self.exp += rhs.exp;
        self.cff *= &rhs.cff;
    }
}

impl DivAssign<&Self> for BigNum {
    fn div_assign(&mut self, rhs: &Self) {
        self.sgn ^= rhs.sgn;
        let (q, r) = self.cff.div_rem(&rhs.cff);
        self.cff = q;
        if r.is_zero() {
            if self.exp > rhs.exp {
                self.exp -= rhs.exp;
            } else {
                let shift = rhs.exp - self.exp;
                self.cff.bits <<= shift;
                self.exp = 0;
            }
            return;
        }
        let mut d = r.div_decimal(&rhs.cff, 15);
        while d.bits.get(0) == 0 {
            d.bits >>= 1;
            d.base -= 1;
        }
        self.cff.bits <<= d.base;
        self.cff.bits |= &d.bits;
        if self.exp + d.base > rhs.exp {
            self.exp += d.base;
            self.exp -= rhs.exp;
        } else {
            let shift = rhs.exp - self.exp - d.base;
            self.cff.bits <<= shift;
            self.exp = 0;
        }
    }
}

macro_rules! impl_oper {
    ($trait:ident, $fn:ident, $oper:tt, $oper_assign:tt) => {
        impl $trait<&Self> for BigNum {
            type Output = BigNum;
            fn $fn(mut self, rhs: &Self) -> Self::Output {
                self $oper_assign rhs;
                self
            }
        }
        impl $trait for &BigNum {
            type Output = BigNum;
            fn $fn(self, rhs: Self) -> Self::Output {
                self.clone() $oper rhs
            }
        }
    };
}
impl_oper!(Add, add, +, +=);
impl_oper!(Sub, sub, -, -=);
impl_oper!(Mul, mul, *, *=);
impl_oper!(Div, div, /, /=);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_float_to_str() {
        let a = BigNum::from(1_u32);
        let b = BigNum::from(-1.234_f32);
        let pi = BigNum::from(std::f64::consts::PI);
        eprintln!("{:?}", b);
        assert_eq!(a.float_to_str(5), "1.00000");
        assert_eq!(b.float_to_str(5), "-1.23399");
        assert_eq!(pi.float_to_str(15), "3.141592653589792");
    }

    #[test]
    fn eq() {
        let a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 1,
        };
        let c = BigNum {
            sgn: 1,
            cff: BigUint::from(2_u32),
            exp: 1,
        };
        let d = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
        };
        let e = BigNum {
            sgn: 0,
            cff: BigUint::from(3_u32),
            exp: 0,
        };
        assert_eq!(a == b, true);
        assert_eq!(a == c, false);
        assert_eq!(a == d, true);
        assert_eq!(a == d, true);
        assert_eq!(a == e, false);

        let pos_z = BigNum {
            sgn: 0,
            cff: BigUint::from(0_u32),
            exp: 0,
        };
        let neg_z = BigNum {
            sgn: 0,
            cff: BigUint::from(0_u32),
            exp: 0,
        };
        assert_eq!(pos_z == neg_z, true);
    }

    #[test]
    fn ord() {
        let a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 0,
        };
        let c = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 5,
        };
        let d = BigNum {
            sgn: 1,
            cff: BigUint::from(1000_u32),
            exp: 0,
        };
        assert_eq!(a < b, true);
        assert_eq!(a > b, false);
        assert_eq!(a < c, false);
        assert_eq!(a > c, true);
        assert_eq!(a < d, false);
        assert_eq!(a > d, true);

        let pos_z = BigNum {
            sgn: 0,
            cff: BigUint::from(0_u32),
            exp: 0,
        };
        let neg_z = BigNum {
            sgn: 0,
            cff: BigUint::from(0_u32),
            exp: 0,
        };
        assert_eq!(pos_z <= neg_z, true);
        assert_eq!(pos_z >= neg_z, true);
    }

    #[test]
    fn add_assign() {
        let mut a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
        };
        let c = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 1,
        };
        let d = BigNum {
            sgn: 1,
            cff: BigUint::from(2_u32),
            exp: 1,
        };
        let e = BigNum {
            sgn: 1,
            cff: BigUint::from(3_u32),
            exp: 0,
        };
        a += &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(2_u32),
                exp: 0
            }
        );
        a += &c;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(6_u32),
                exp: 1
            }
        );
        a += &d;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(4_u32),
                exp: 1
            }
        );
        a += &e;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(1_u32),
                exp: 0
            }
        );
    }

    #[test]
    fn sub_assign() {
        let mut a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
        };
        let c = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 1,
        };
        let d = BigNum {
            sgn: 1,
            cff: BigUint::from(2_u32),
            exp: 1,
        };
        a -= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(0_u32),
                exp: 0
            }
        );
        a -= &c;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(2_u32),
                exp: 1
            }
        );
        a -= &d;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(0_u32),
                exp: 1
            }
        );
    }

    #[test]
    fn mul() {
        let mut a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 0,
        };
        let c = BigNum {
            sgn: 1,
            cff: BigUint::from(3_u32),
            exp: 0,
        };
        let d = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 3,
        };
        a *= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(2_u32),
                exp: 0
            }
        );
        a *= &c;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(6_u32),
                exp: 0
            }
        );
        a *= &d;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(6_u32),
                exp: 3
            }
        );
    }

    #[test]
    fn div_assign() {
        let mut a = BigNum {
            sgn: 0,
            cff: BigUint::from(27_u32),
            exp: 1,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(3_u32),
            exp: 0,
        };
        let c = BigNum {
            sgn: 1,
            cff: BigUint::from(3_u32),
            exp: 0,
        };
        let d = BigNum {
            sgn: 1,
            cff: BigUint::from(3_u32),
            exp: 1,
        };
        let e = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 3,
        };
        a /= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(9_u32),
                exp: 1
            }
        );
        a /= &c;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(3_u32),
                exp: 1
            }
        );
        a /= &d;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(1_u32),
                exp: 0
            }
        );
        a /= &e;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(8_u32),
                exp: 0
            }
        );
    }

    #[test]
    fn macro_impl_oper() {
        let n = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 1,
        };
        assert_eq!(
            &n + &n + &n,
            BigNum {
                sgn: 0,
                cff: BigUint::from(3_u32),
                exp: 1
            }
        );
        assert_eq!(
            &n - &n - &n,
            BigNum {
                sgn: 1,
                cff: BigUint::from(1_u32),
                exp: 1
            }
        );
        assert_eq!(
            &n * &n * &n,
            BigNum {
                sgn: 0,
                cff: BigUint::from(1_u32),
                exp: 3
            }
        );
        assert_eq!(
            &n / &n / &n,
            BigNum {
                sgn: 0,
                cff: BigUint::from(2_u32),
                exp: 0
            }
        );

        let deci = BigNum {
            sgn: 0,
            cff: BigUint::from(3_u32),
            exp: 0,
        };
        assert_eq!(
            &n / &n / &deci,
            BigNum {
                sgn: 0,
                cff: BigUint::from(1501199875790165_u64),
                exp: 52
            }
        );
    }
}
