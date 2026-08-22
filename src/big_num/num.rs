#![allow(unused)]

use super::uint::BigUint;
use std::cmp::{Ord, PartialOrd};
use std::convert::{From, TryFrom, TryInto};
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

const IRR_COND: usize = (u64::BITS * 10) as usize;

#[derive(Clone)]
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

impl BigNum {
    pub fn is_int(&self) -> bool {
        (&self.num % &self.den).is_zero()
    }
    pub fn is_inf(&self) -> bool {
        self.den.is_zero()
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
                res.sgn = sgn;
                res
            }
        }
    };
}
impl_from_float!(f32, 7);
impl_from_float!(f64, 15);

impl From<BigUint> for BigNum {
    fn from(value: BigUint) -> Self {
        Self {
            sgn: 0,
            num: value,
            den: BigUint::from(1_u32),
            irr: true,
        }
    }
}

impl TryFrom<&str> for BigNum {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        fn is_number(s: &str) -> bool {
            s.chars().all(|c| c.is_digit(10))
        }

        if value.is_empty() {
            return Err(());
        }
        let sgn = if value.chars().nth(0).unwrap() == '-' {
            1
        } else {
            0
        };
        let mut sci_split = (if sgn == 1 { &value[1..] } else { value }).splitn(2, ['e', 'E']);

        let frac = sci_split.next().ok_or(())?;
        let mut frac_split = frac.splitn(2, '.');
        let int = frac_split.next().ok_or(())?;
        if !is_number(int) {
            return Err(());
        }
        let int = BigUint::from(int);
        let dec = if let Some(dec) = frac_split.next() {
            let exp = dec.len();
            if !is_number(dec) {
                return Err(());
            }
            let mut dec = BigUint::from(dec);
            (dec, exp)
        } else {
            (BigUint::new(), 0)
        };

        let exp10 = if let Some(exp) = sci_split.next() {
            if let Some(sidx) = exp.find(['+', '-']) {
                if sidx != 0 {
                    return Err(());
                }
                if !is_number(&exp[1..]) {
                    return Err(());
                }
                if exp.chars().nth(0).unwrap() == '+' {
                    (BigUint::from(&exp[1..]), 0_u8)
                } else {
                    (BigUint::from(&exp[1..]), 1_u8)
                }
            } else {
                (BigUint::from(exp), 0_u8)
            }
        } else {
            (BigUint::new(), 0_u8)
        };

        let int = {
            let mut int = BigNum::from(int);
            int.sgn = sgn;
            int
        };
        let dec = {
            let mut exp = dec.1;
            let mut base = BigUint::from(10_u32);
            let mut den = BigUint::from(1_u32);
            while exp > 0 {
                if exp & 1 == 1 {
                    den *= &base;
                }
                base *= &base.clone();
                exp >>= 1;
            }
            BigNum {
                sgn,
                num: dec.0,
                den,
                irr: false,
            }
        };
        let exp10 = {
            let mut exp = exp10.0;
            let mut base = BigUint::from(10_u32);
            let mut pow = BigUint::from(1_u32);
            let one = BigUint::from(1_u32);
            let two = BigUint::from(2_u32);
            while !exp.is_zero() {
                if &exp % &two == one {
                    pow *= &base;
                }
                base *= &base.clone();
                exp /= &two;
            }
            if exp10.1 == 0 {
                BigNum {
                    sgn: 0,
                    num: pow,
                    den: one,
                    irr: true,
                }
            } else {
                BigNum {
                    sgn: 0,
                    num: one,
                    den: pow,
                    irr: true,
                }
            }
        };
        Ok((&int + &dec) * &exp10)
    }
}

impl TryFrom<String> for BigNum {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
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
        if self.num.bit_capacity() >= IRR_COND || self.den.bit_capacity() >= IRR_COND {
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

impl Ord for BigNum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        let mut s = self.clone();
        let mut o = other.clone();
        Self::common_both(&mut s, &mut o);
        match s.sgn.cmp(&o.sgn) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => {
                if s.sgn == 0 {
                    s.num.cmp(&o.num)
                } else {
                    match s.num.cmp(&o.num) {
                        Ordering::Less => Ordering::Greater,
                        Ordering::Greater => Ordering::Less,
                        Ordering::Equal => Ordering::Equal,
                    }
                }
            }
        }
    }
}

impl PartialOrd for BigNum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

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
            write!(
                f,
                "({})({})/({})",
                if self.sgn == 0 { "+" } else { "-" },
                self.num,
                self.den
            )
        }
    }
}

impl Debug for BigNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = self.clone();
        s.reduce();
        write!(f, "{}", s)
    }
}

impl BigNum {
    pub fn to_float_str(&self, precision: u8) -> String {
        if self.den.is_zero() {
            return format!("{}{}", if self.sgn == 0 { "+" } else { "-" }, "INF");
        }
        let (q, r) = self.num.div_rem(&self.den);
        let d = r.div_decimal(&self.den, precision);
        if self.sgn == 0 {
            format!("{}.{}", q, d)
        } else {
            format!("-{}.{}", q, d)
        }
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
    fn cmp() {
        let a = BigNum::from(1_u32);
        let b = BigNum::from(2_u32);
        let c = BigNum::from(-2_i32);
        let d = BigNum {
            sgn: 0,
            num: BigUint::from(4_u32),
            den: BigUint::from(2_u32),
            irr: false,
        };
        assert_eq!(a > b, false);
        assert_eq!(a >= b, false);
        assert_eq!(a < b, true);
        assert_eq!(a <= b, true);
        assert_eq!(a > c, true);
        assert_eq!(a >= c, true);
        assert_eq!(a < c, false);
        assert_eq!(a <= c, false);
        assert_eq!(a > d, false);
        assert_eq!(a >= d, false);
        assert_eq!(a < d, true);
        assert_eq!(a <= d, true);
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
    fn try_from_str() {
        let int_pos = BigNum::try_from("10").unwrap();
        let int_neg = BigNum::try_from("-10").unwrap();
        assert_eq!(int_pos, BigNum::from(10));
        assert_eq!(int_neg, BigNum::from(-10));

        let dec = BigNum::try_from("1.234").unwrap();
        assert_eq!(dec, BigNum::from(1.234_f64));

        let sciu = BigNum::try_from("1.234E3").unwrap();
        let scil = BigNum::try_from("1.234e3").unwrap();
        let sci_epos = BigNum::try_from("1.234e+3").unwrap();
        let sci_eneg = BigNum::try_from("1.234e-3").unwrap();
        assert_eq!(sciu, BigNum::from(1.234E3_f64));
        assert_eq!(scil, BigNum::from(1.234e3_f64));
        assert_eq!(sci_epos, BigNum::from(1.234e+3_f64));
        assert_eq!(sci_eneg, BigNum::from(1.234e-3_f64));

        let int_sci = BigNum::try_from("1e5").unwrap();
        assert_eq!(int_sci, BigNum::from(100000_u32));

        let full = BigNum::try_from("-1.234E+3").unwrap();
        assert_eq!(full, BigNum::from(-1.234E+3_f64));
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
