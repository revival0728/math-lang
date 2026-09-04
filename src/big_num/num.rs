use super::uint::BigUint;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use std::convert::{From, TryFrom};
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Clone)]
/// Structure stores big numbers in form of `(+/-) cff * 2^(-exp)`
pub struct BigNum {
    sgn: u8, // 0: positive, 1: negative
    cff: BigUint,
    exp: u32,
    inf: i8, // 0: Not INF, 1: +INF, -1: -INF, ignore self.sgn
    nan: bool,
}

impl BigNum {
    pub fn new() -> Self {
        Self {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: 0,
            nan: false,
        }
    }
    pub fn inf() -> Self {
        Self {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: 1,
            nan: false,
        }
    }
    pub fn neg_inf() -> Self {
        Self {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: -1,
            nan: false,
        }
    }
    pub fn nan() -> Self {
        Self {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: -1,
            nan: true,
        }
    }
}

/// math functions
impl BigNum {
    fn ilog2(&self) -> Self {
        let mut lg = Self::new();
        lg.cff.bits.set(0);
        lg.cff.bits <<= self.cff.bit_count();
        lg
    }
    pub fn trunc(&self) -> Self {
        let mut ret = self.clone();
        ret.cff.bits >>= ret.exp;
        ret.exp = 0;
        ret
    }
    pub fn fract(&self) -> Self {
        let mut ret = BigNum::new();
        ret.sgn = self.sgn;
        ret.nan = self.nan;
        ret.inf = self.inf;
        ret.exp = self.exp;
        for i in 0..self.exp as usize {
            if self.cff.bits.get(i) == 1 {
                ret.cff.bits.set(i);
            }
        }
        ret
    }
    pub fn abs(&self) -> Self {
        if !self.is_finite_number() {
            return Self::nan();
        }
        let mut ret = self.clone();
        ret.sgn = 0;
        ret.inf = ret.inf.abs();
        ret
    }
    pub fn sqrt(&self) -> Self {
        if !self.is_finite_number() || self.sgn == 1 {
            return Self::nan();
        }
        let mut n = self.ilog2();
        let two_reci = Self::from(0.5);
        for _ in 0..self.cff.bit_count() + 15 {
            n = &two_reci * &(&n + &(self / &n));
        }
        n
    }
    pub fn exp2(&self) -> Self {
        if !self.is_finite_number() {
            return Self::nan();
        }
        let int = {
            let mut int = BigNum::from(1);
            let mut exp = self.clone();
            let mut pow = BigNum::from(2);
            exp.cff.bits >>= exp.exp;
            exp.exp = 0;
            while !exp.cff.bits.all_zero() {
                if exp.cff.bits.get(0) == 1 {
                    int *= &pow;
                }
                pow.cff.bits <<= pow.cff.bits.bit_len() - 1;
                exp.cff.bits >>= 1;
            }
            int
        };
        let dec = {
            let mut dec_exp = self.fract();
            dec_exp.trunc_with_precision(1020);
            let dec_exp = dec_exp.to_f64_unchecked();
            BigNum::from(2_f64.powf(dec_exp))
        };
        if self.sgn == 0 {
            int * &dec
        } else {
            BigNum::from(1) / &(int * &dec)
        }
    }
    pub fn exp(&self) -> Self {
        if !self.is_finite_number() {
            return Self::nan();
        }
        let lg2e = BigNum::from(std::f64::consts::LOG2_E);
        (self * &lg2e).exp2()
    }
    /// scale cff part down to 1.xxx...
    /// return `(BigNum with same value, (shift exp sign, shift exp value))`
    fn log_sdn(&self) -> (Self, (u8, u32)) {
        let bl = self.cff.bits.bit_len() as u32;
        let exp = {
            if bl - 1 >= self.exp {
                (0_u8, bl - self.exp - 1)
            } else {
                (1_u8, self.exp - bl + 1)
            }
        };
        let mut m = self.clone();
        if exp.0 == 0 {
            m.exp += exp.1
        } else {
            m.exp -= exp.1;
        }
        (m, exp)
    }
    pub fn ln(&self) -> Self {
        if !self.is_finite_number() || self.sgn == 1 {
            return Self::nan();
        }
        use std::f64::consts::LN_2;
        let (m, exp) = self.log_sdn();
        let m = m.to_f64_unchecked();
        let log_cff = BigNum::from(m.ln());
        let log_exp = BigNum::from(exp.1) * &BigNum::from(LN_2);
        if exp.0 == 0 {
            log_cff + &log_exp
        } else {
            log_cff - &log_exp
        }
    }
    pub fn log2(&self) -> Self {
        if !self.is_finite_number() || self.sgn == 1 {
            return Self::nan();
        }
        let (m, exp) = self.log_sdn();
        let m = m.to_f64_unchecked();
        let log_cff = BigNum::from(m.log2());
        let log_exp = BigNum::from(exp.1);
        if exp.0 == 0 {
            log_cff + &log_exp
        } else {
            log_cff - &log_exp
        }
    }
    pub fn log10(&self) -> Self {
        if !self.is_finite_number() || self.sgn == 1 {
            return Self::nan();
        }
        use std::f64::consts::LOG10_2;
        let (m, exp) = self.log_sdn();
        let m = m.to_f64_unchecked();
        let log_cff = BigNum::from(m.log10());
        let log_exp = BigNum::from(exp.1) * &BigNum::from(LOG10_2);
        if exp.0 == 0 {
            log_cff + &log_exp
        } else {
            log_cff - &log_exp
        }
    }
    pub fn pow(&self, exp: &BigNum) -> Self {
        if !self.is_finite_number() {
            return Self::nan();
        }
        (exp * &self.log2()).exp2()
    }
    /// scale down self into range `[0, 2*PI)`
    fn trig_sdn(&self) -> Self {
        use std::f64::consts::PI;
        let two_pi = BigNum::from(PI + PI);
        let k = (self / &two_pi).trunc();
        let r = self - &(&k * &two_pi);
        r
    }
    pub fn sin(&self) -> Self {
        if !self.is_finite_number() {
            return Self::nan();
        }
        let theta = self.trig_sdn().to_f64_unchecked();
        BigNum::from(theta.sin())
    }
    pub fn cos(&self) -> Self {
        if !self.is_finite_number() {
            return Self::nan();
        }
        let theta = self.trig_sdn().to_f64_unchecked();
        BigNum::from(theta.cos())
    }
    pub fn tan(&self) -> Self {
        if !self.is_finite_number() {
            return Self::nan();
        }
        let theta = self.trig_sdn().to_f64_unchecked();
        BigNum::from(theta.tan())
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
                    inf: 0,
                    nan: false,
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
                    inf: 0,
                    nan: false,
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
            inf: 0,
            nan: false,
        }
    }
}

macro_rules! impl_from_float {
    ($type:ty, $prec:literal) => {
        impl From<$type> for BigNum {
            /// only support value ranges in `[-2^128, 2^128]`
            fn from(value: $type) -> Self {
                const PRECISION: u8 = $prec;
                let sgn = if value.signum() < 0.0 { 1 } else { 0 };
                if value.is_infinite() {
                    let mut res = Self::inf();
                    res.sgn ^= sgn;
                    return res;
                }
                if value.is_nan() {
                    return Self::nan();
                }
                let value = value.abs();
                let int = unsafe { value.trunc().to_int_unchecked::<u128>() };
                let dec = unsafe {
                    (value.fract() * (10 as $type).powi(PRECISION as i32) as $type)
                        .trunc()
                        .to_int_unchecked::<u64>()
                };
                let p10 = if PRECISION > 15 {
                    BigNum::from((10_u64).pow((PRECISION - 15_u8) as u32))
                        * &BigNum::from((10_u64).pow(15_u8 as u32))
                } else {
                    BigNum::from((10_u64).pow(PRECISION as u32))
                };

                let int = BigNum::from(int);
                let dec = BigNum::from(dec);
                let mut ret = int + &(&dec / &p10);
                ret.sgn = sgn;
                ret
            }
        }
    };
}
impl_from_float!(f32, 7);
impl_from_float!(f64, 19);

/// From scientific notation
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
            let dec = BigUint::from(dec);
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
            let mut p10 = BigUint::from(1_u32);
            while exp > 0 {
                if exp & 1 == 1 {
                    p10 *= &base;
                }
                base *= &base.clone();
                exp >>= 1;
            }
            let mut dec = BigNum::from(dec.0)
                .div_with_precision(&BigNum::from(p10), (dec.1 + 15).min(u8::MAX as usize) as u8);
            dec.sgn = sgn;
            dec
        };
        let exp10 = {
            let prec = if exp10.0 <= BigUint::from(255_u8 - 15_u8) {
                u8::from_le_bytes([exp10.0.bits.to_le_bytes()[0]]) + 15_u8
            } else {
                255_u8
            };
            let mut exp = exp10.0;
            let mut base = BigUint::from(10_u32);
            let mut pow = BigUint::from(1_u32);
            let one = BigUint::from(1_u32);
            while !exp.is_zero() {
                if exp.bits.get(0) == 1 {
                    pow *= &base;
                }
                base *= &base.clone();
                exp.bits >>= 1;
            }
            if exp10.1 == 0 {
                BigNum::from(pow)
            } else {
                BigNum::from(one).div_with_precision(&BigNum::from(pow), prec)
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

impl Display for BigNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_float_str(15))
    }
}

impl Debug for BigNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.nan {
            write!(f, "{}", "NaN")
        } else if self.inf != 0 {
            use std::cmp::Ordering::{Equal, Greater, Less};
            write!(
                f,
                "{}",
                match self.inf.cmp(&0) {
                    Less => "-INF",
                    Greater => "+INF",
                    Equal => panic!("IMPOSSIBLE"),
                }
            )
        } else {
            write!(
                f,
                "({})({})*(2^(-{}))",
                if self.sgn == 0 { "+" } else { "-" },
                self.cff,
                self.exp
            )
        }
    }
}

impl BigNum {
    pub fn to_float_str(&self, precision: u8) -> String {
        if self.nan {
            "NaN".to_owned()
        } else if self.inf != 0 {
            use std::cmp::Ordering::{Equal, Greater, Less};
            match self.inf.cmp(&0) {
                Less => "-INF",
                Greater => "+INF",
                Equal => panic!("IMPOSSIBLE"),
            }
            .to_owned()
        } else {
            let pow2 = BigUint::pow2(self.exp);
            let (int, dec) = self.cff.div_rem(&pow2);
            let dec = dec.div_decimal(&pow2, precision);
            format!("{}{}.{}", if self.sgn == 1 { "-" } else { "" }, int, dec)
        }
    }
}

impl PartialEq for BigNum {
    fn eq(&self, other: &Self) -> bool {
        if self.nan || other.nan {
            return self.nan && other.nan;
        }
        if self.inf != 0 || other.inf != 0 {
            return self.inf == other.inf;
        }
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
        if self.nan || other.nan {
            return Equal;
        }
        if self.inf != 0 || other.inf != 0 {
            return self.inf.cmp(&other.inf);
        }
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
    fn align_exp(a: &mut Self, b: &mut Self) {
        if a.exp == b.exp {
            return;
        }
        let target = std::cmp::max(a.exp, b.exp);
        a.cff.bits <<= target - a.exp;
        b.cff.bits <<= target - b.exp;
        a.exp = target;
        b.exp = target;
    }
    pub fn is_finite_number(&self) -> bool {
        !self.nan && self.inf == 0
    }
    pub fn trunc_with_precision(&mut self, base2: u32) {
        let trunc = std::cmp::min(self.exp, base2);
        self.cff.bits >>= self.exp - trunc;
        self.exp = trunc;
    }
    pub fn to_f64_unchecked(&self) -> f64 {
        if self.nan {
            return f64::NAN;
        }
        if self.inf == 1 {
            return f64::INFINITY;
        }
        if self.inf == -1 {
            return f64::NEG_INFINITY;
        }
        let mut b2 = 2_f64.powi(-(self.exp as i32));
        self.cff
            .bits
            .iter()
            .fold(0_f64, |mut r, bit| {
                r += b2 * bit as f64;
                b2 *= 2.0;
                return r;
            })
            .copysign(if self.sgn == 0 { 1.0 } else { -1.0 })
    }
}

impl Neg for &BigNum {
    type Output = BigNum;
    fn neg(self) -> Self::Output {
        let mut ret = self.clone();
        if ret.inf != 0 {
            ret.inf *= -1;
        } else {
            ret.sgn ^= 1;
        }
        ret
    }
}

impl AddAssign<&Self> for BigNum {
    fn add_assign(&mut self, rhs: &Self) {
        if self.nan || rhs.nan {
            self.nan = true;
            return;
        }
        if self.inf != 0 || rhs.inf != 0 {
            self.inf = (self.inf + rhs.inf).signum();
            if self.inf == 0 {
                self.sgn = 0;
                self.cff = BigUint::from(0_u32);
                self.exp = 0;
            }
            return;
        }
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
        if self.nan || rhs.nan {
            self.nan = true;
            return;
        }
        if self.inf != 0 || rhs.inf != 0 {
            self.inf = (self.inf - rhs.inf).signum();
            if self.inf == 0 {
                self.sgn = 0;
                self.cff = BigUint::from(0_u32);
                self.exp = 0;
            }
            return;
        }
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
        if self.nan || rhs.nan {
            self.nan = true;
            return;
        }
        if self.inf != 0 && rhs.inf != 0 {
            self.inf = (self.inf * rhs.inf).signum();
            return;
        }
        if self.inf != 0 || rhs.inf != 0 {
            if self.cff.is_zero() && self.inf == 0 {
                return;
            }
            if rhs.cff.is_zero() && rhs.inf == 0 {
                self.sgn = 0;
                self.cff = BigUint::new();
                self.exp = 0;
                self.inf = 0;
                return;
            }
            if self.inf != 0 {
                self.inf *= if rhs.sgn == 0 { 1 } else { -1 };
                return;
            }
            if rhs.inf != 0 {
                self.inf = rhs.inf;
                self.inf *= if self.sgn == 0 { 1 } else { -1 };
                return;
            }
        }
        self.sgn ^= rhs.sgn;
        self.exp += rhs.exp;
        self.cff *= &rhs.cff;
    }
}

impl BigNum {
    pub fn div_assign_with_precision(&mut self, rhs: &Self, precision: u8) {
        if self.nan || rhs.nan {
            self.nan = true;
            return;
        }
        if self.inf != 0 && rhs.inf != 0 {
            self.sgn = if self.inf * rhs.inf >= 0 { 0 } else { 1 };
            self.cff = BigUint::from(1_u32);
            self.exp = 0;
            self.inf = 0;
            return;
        }
        if self.inf != 0 || rhs.inf != 0 {
            if rhs.cff.is_zero() && rhs.inf == 0 {
                return;
            }
            if self.inf != 0 {
                self.inf *= if rhs.sgn == 0 { 1 } else { -1 };
                return;
            }
            if rhs.inf != 0 {
                self.sgn = 0;
                self.cff = BigUint::new();
                self.exp = 0;
                self.inf = 0;
                return;
            }
        }
        if rhs.cff.is_zero() {
            if self.cff.is_zero() {
                self.nan = true;
                return;
            }
            self.inf = if self.sgn ^ rhs.sgn == 0 { 1 } else { -1 };
            return;
        }
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
        let mut d = r.div_decimal(&rhs.cff, precision);
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
    pub fn div_with_precision(&self, rhs: &Self, precision: u8) -> Self {
        let mut ret = self.clone();
        ret.div_assign_with_precision(rhs, precision);
        ret
    }
}

impl DivAssign<&Self> for BigNum {
    fn div_assign(&mut self, rhs: &Self) {
        self.div_assign_with_precision(rhs, 15);
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
    fn fract() {
        let a = 0.5_f64;
        let b = 1.234_f64;
        let c = 12345.6789_f64;
        assert_eq!(BigNum::from(a).fract(), BigNum::from(a.fract()));
        assert_eq!(BigNum::from(b).fract(), BigNum::from(b.fract()));
        assert_eq!(BigNum::from(c).fract(), BigNum::from(c.fract()));
    }

    #[test]
    fn to_f64_unchecked() {
        fn check_eq(lhs: f64, rhs: f64) {
            const EPS: f64 = 1e-15;
            assert!((lhs - rhs).abs() < EPS);
        }
        use std::f64::consts::PI;
        let a = 0.5_f64;
        let b = 1.234_f64;
        let c = 12345.6789_f64;
        check_eq(BigNum::from(a).to_f64_unchecked(), a);
        check_eq(BigNum::from(b).to_f64_unchecked(), b);
        check_eq(BigNum::from(c).to_f64_unchecked(), c);
        check_eq(BigNum::from(PI).to_f64_unchecked(), PI);
    }

    #[test]
    fn sqrt() {
        let eps = BigNum::from(1e-15_f64);

        let two = BigNum::from(2);
        assert!((&two.sqrt() - &BigNum::from(2_f64.sqrt())).abs() < eps);
        let normal = BigNum::from(12345);
        assert!((&normal.sqrt() - &BigNum::from(12345_f64.sqrt())).abs() < eps);
        let large = BigNum::from(1e18_f64);
        assert!((&large.sqrt() - &BigNum::from(1e18_f64.sqrt())).abs() < eps);
    }

    #[test]
    fn exp2() {
        let eps = BigNum::from(1e-15_f64);

        let a = BigNum::from(2);
        assert!((&a.exp2() - &BigNum::from(2_f64.powi(2))).abs() < eps);
        let b = BigNum::from(0.5);
        assert!((&b.exp2() - &BigNum::from(2_f64.powf(0.5))).abs() < eps);
        let c = BigNum::from(1.125);
        assert!((&c.exp2() - &BigNum::from(2_f64.powf(1.125))).abs() < eps);
    }

    #[test]
    fn exp() {
        let eps = BigNum::from(1e-14_f64);

        let a = BigNum::from(2);
        assert!((&a.exp() - &BigNum::from(2_f64.exp())).abs() < eps);
        let b = BigNum::from(0.5);
        assert!((&b.exp() - &BigNum::from(0.5_f64.exp())).abs() < eps);
        let c = BigNum::from(1.125);
        assert!((&c.exp() - &BigNum::from(1.125_f64.exp())).abs() < eps);
    }

    #[test]
    fn ln() {
        let eps = BigNum::from(1e-15_f64);

        let a = BigNum::from(2);
        assert!((&a.ln() - &BigNum::from(2_f64.ln())).abs() < eps);
        let b = BigNum::from(1e18_f64);
        assert!((&b.ln() - &BigNum::from(1e18_f64.ln())).abs() < eps);
        let c = BigNum::from(std::f64::consts::E);
        assert!((&c.ln() - &BigNum::from(std::f64::consts::E.ln())).abs() < eps);
        let large =
            BigNum::try_from("1123987230502758902374987198273981729472398582634672").unwrap();
        assert_eq!(large.ln().to_float_str(15), "117.548722133340621");
    }

    #[test]
    fn log2() {
        let eps = BigNum::from(1e-14_f64);

        let a = BigNum::from(2);
        assert!((&a.log2() - &BigNum::from(2_f64.log2())).abs() < eps);
        let b = BigNum::from(1e18_f64);
        assert!((&b.log2() - &BigNum::from(1e18_f64.log2())).abs() < eps);
        let c = BigNum::from(1);
        assert!((&c.log2() - &BigNum::from(1_f64.log2())).abs() < eps);
    }

    #[test]
    fn log10() {
        let eps = BigNum::from(1e-14_f64);

        let a = BigNum::from(2);
        assert!((&a.log10() - &BigNum::from(2_f64.log10())).abs() < eps);
        let b = BigNum::from(1e18_f64);
        assert!((&b.log10() - &BigNum::from(1e18_f64.log10())).abs() < eps);
        let c = BigNum::from(10);
        assert!((&c.log10() - &BigNum::from(10_f64.log10())).abs() < eps);
    }

    #[test]
    fn pow() {
        let eps = BigNum::from(1e-15_f64);

        let a_base = BigNum::from(2);
        let a_exp = BigNum::from(5.283);
        assert!((&a_base.pow(&a_exp) - &BigNum::from(2_f64.powf(5.283))).abs() < eps);
        let b_base = BigNum::from(1e18_f64);
        let b_exp = BigNum::from(0.0472);
        assert!((&b_base.pow(&b_exp) - &BigNum::from(1e18_f64.powf(0.0472))).abs() < eps);
    }

    #[test]
    fn sin() {
        use std::f64::consts::FRAC_PI_2;
        use std::f64::consts::PI;
        let eps = BigNum::from(1e-15_f64);

        let a = BigNum::from(0_f64);
        assert!((&a - &BigNum::from(0_f64.sin())).abs() < eps);
        let b = BigNum::from(FRAC_PI_2);
        assert!((&b.sin() - &BigNum::from(FRAC_PI_2.sin())).abs() < eps);
        let c = BigNum::from(PI);
        assert!((&c.sin() - &BigNum::from(PI.sin())).abs() < eps);
        let d = BigNum::from(-1.234_f64);
        assert!((&d.sin() - &BigNum::from((-1.234_f64).sin())).abs() < eps);
        let e = BigNum::from(PI + PI);
        assert!((&e.sin() - &BigNum::from((PI + PI).sin())).abs() < eps);
        let g = BigNum::from(FRAC_PI_2 + PI);
        assert!((&g.sin() - &BigNum::from((FRAC_PI_2 + PI).sin())).abs() < eps);

        let eps_tor = BigNum::from(1e-13_f64);
        let f = BigNum::from(-1234_f64);
        assert!((&f.sin() - &BigNum::from((-1234_f64).sin())).abs() < eps_tor);
    }

    #[test]
    fn cos() {
        use std::f64::consts::FRAC_PI_2;
        use std::f64::consts::PI;
        let eps = BigNum::from(1e-15_f64);

        let a = BigNum::from(0_f64);
        assert!((&a.cos() - &BigNum::from(0_f64.cos())).abs() < eps);
        let b = BigNum::from(FRAC_PI_2);
        assert!((&b.cos() - &BigNum::from(FRAC_PI_2.cos())).abs() < eps);
        let c = BigNum::from(PI);
        assert!((&c.cos() - &BigNum::from(PI.cos())).abs() < eps);
        let d = BigNum::from(-1.234_f64);
        assert!((&d.cos() - &BigNum::from((-1.234_f64).cos())).abs() < eps);
        let e = BigNum::from(PI + PI);
        assert!((&e.cos() - &BigNum::from((PI + PI).cos())).abs() < eps);
        let g = BigNum::from(FRAC_PI_2 + PI);
        eprintln!("{}, {}", g.cos(), BigNum::from((FRAC_PI_2 + PI).cos()));
        assert!((&g.cos() - &BigNum::from((FRAC_PI_2 + PI).cos())).abs() < eps);

        let eps_tor = BigNum::from(1e-13_f64);
        let f = BigNum::from(-1234_f64);
        assert!((&f.cos() - &BigNum::from((-1234_f64).cos())).abs() < eps_tor);
    }

    #[test]
    fn tan() {
        use std::f64::consts::FRAC_PI_2;
        use std::f64::consts::PI;
        let eps = BigNum::from(1e-15_f64);

        let a = BigNum::from(0_f64);
        assert!((&a.tan() - &BigNum::from(0_f64.tan())).abs() < eps);
        let b = BigNum::from(FRAC_PI_2);
        assert!((&b.tan() - &BigNum::from(FRAC_PI_2.tan())).abs() < eps);
        let c = BigNum::from(PI);
        assert!((&c.tan() - &BigNum::from(PI.tan())).abs() < eps);
        let d = BigNum::from(-1.234_f64);
        assert!((&d.tan() - &BigNum::from((-1.234_f64).tan())).abs() < eps);
        let e = BigNum::from(PI + PI);
        assert!((&e.tan() - &BigNum::from((PI + PI).tan())).abs() < eps);
        let g = BigNum::from(FRAC_PI_2 + PI);
        assert!((&g.tan() - &BigNum::from((FRAC_PI_2 + PI).tan())).abs() < eps);

        let eps_tor = BigNum::from(1e-13_f64);
        let f = BigNum::from(-1234_f64);
        assert!((&f.tan() - &BigNum::from((-1234_f64).tan())).abs() < eps_tor);
    }

    #[test]
    fn from_float_to_str() {
        let a = BigNum::from(1_u32);
        let b = BigNum::from(-1.234_f32);
        let c = BigNum::from(1.234e-3_f64);
        let pi = BigNum::from(std::f64::consts::PI);
        assert_eq!(a.to_float_str(5), "1.00000");
        assert_eq!(b.to_float_str(5), "-1.23399");
        assert_eq!(c.to_float_str(7), "0.0012339");
        assert_eq!(pi.to_float_str(15), "3.141592653589793");
    }

    #[test]
    fn try_from_str() {
        let eps = BigNum::from(1e-15_f64);

        let int_pos = BigNum::try_from("10").unwrap();
        let int_neg = BigNum::try_from("-10").unwrap();
        assert_eq!(int_pos, BigNum::from(10));
        assert_eq!(int_neg, BigNum::from(-10));

        let dec = BigNum::try_from("1.234").unwrap();
        assert!((&dec - &BigNum::from(1.234_f64)).abs() < eps);

        let sciu = BigNum::try_from("1.234E3").unwrap();
        let scil = BigNum::try_from("1.234e3").unwrap();
        let sci_epos = BigNum::try_from("1.234e+3").unwrap();
        let sci_eneg = BigNum::try_from("1.234e-3").unwrap();
        assert!((&sciu - &BigNum::from(1.234E3_f64)).abs() < eps);
        assert!((&scil - &BigNum::from(1.234e3_f64)).abs() < eps);
        assert!((&sci_epos - &BigNum::from(1.234e+3_f64)).abs() < eps);
        assert!((&sci_eneg - &BigNum::from(1.234e-3_f64)).abs() < eps);

        let int_sci = BigNum::try_from("1e5").unwrap();
        assert!((&int_sci - &BigNum::from(100000_u32)).abs() < eps);

        let full = BigNum::try_from("-1.234E+3").unwrap();
        assert!((&full - &BigNum::from(-1.234E+3_f64)).abs() < eps);
    }

    #[test]
    fn eq() {
        let a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        let c = BigNum {
            sgn: 1,
            cff: BigUint::from(2_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        let d = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let e = BigNum {
            sgn: 0,
            cff: BigUint::from(3_u32),
            exp: 0,
            inf: 0,
            nan: false,
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
            inf: 0,
            nan: false,
        };
        let neg_z = BigNum {
            sgn: 0,
            cff: BigUint::from(0_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        assert_eq!(pos_z == neg_z, true);

        let pos_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: 1,
            nan: false,
        };
        let neg_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: -1,
            nan: false,
        };
        assert_eq!(pos_inf == pos_inf, true);
        assert_eq!(pos_inf != neg_inf, true);
    }

    #[test]
    fn ord() {
        let a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let c = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 5,
            inf: 0,
            nan: false,
        };
        let d = BigNum {
            sgn: 1,
            cff: BigUint::from(1000_u32),
            exp: 0,
            inf: 0,
            nan: false,
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
            inf: 0,
            nan: false,
        };
        let neg_z = BigNum {
            sgn: 0,
            cff: BigUint::from(0_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        assert_eq!(pos_z <= neg_z, true);
        assert_eq!(pos_z >= neg_z, true);

        let pos_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: 1,
            nan: false,
        };
        let neg_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: -1,
            nan: false,
        };
        assert_eq!(pos_inf > pos_inf, false);
        assert_eq!(pos_inf < pos_inf, false);
        assert_eq!(pos_inf < neg_inf, false);
        assert_eq!(pos_inf > neg_inf, true);
    }

    #[test]
    fn neg() {
        let n = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let zero = BigNum::new();
        assert_eq!(n > zero, true);
        let n = -&n;
        assert_eq!(n < zero, true);
    }

    #[test]
    fn add_assign() {
        let mut a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let c = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        let d = BigNum {
            sgn: 1,
            cff: BigUint::from(2_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        let e = BigNum {
            sgn: 1,
            cff: BigUint::from(3_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        a += &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(2_u32),
                exp: 0,
                inf: 0,
                nan: false,
            }
        );
        a += &c;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(6_u32),
                exp: 1,
                inf: 0,
                nan: false,
            }
        );
        a += &d;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(4_u32),
                exp: 1,
                inf: 0,
                nan: false,
            }
        );
        a += &e;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(1_u32),
                exp: 0,
                inf: 0,
                nan: false,
            }
        );

        let pos_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: 1,
            nan: false,
        };
        let neg_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: -1,
            nan: false,
        };
        a += &pos_inf;
        assert_eq!(a, pos_inf);
        a += &pos_inf;
        assert_eq!(a, pos_inf);
        a += &neg_inf;
        assert_eq!(a, BigNum::new());
        a += &neg_inf;
        assert_eq!(a, neg_inf);
    }

    #[test]
    fn sub_assign() {
        let mut a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let c = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        let d = BigNum {
            sgn: 1,
            cff: BigUint::from(2_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        a -= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(0_u32),
                exp: 0,
                inf: 0,
                nan: false,
            }
        );
        a -= &c;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(2_u32),
                exp: 1,
                inf: 0,
                nan: false,
            }
        );
        a -= &d;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(0_u32),
                exp: 1,
                inf: 0,
                nan: false,
            }
        );

        let pos_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: 1,
            nan: false,
        };
        let neg_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: -1,
            nan: false,
        };
        a -= &pos_inf;
        assert_eq!(a, neg_inf);
        a -= &pos_inf;
        assert_eq!(a, neg_inf);
        a -= &neg_inf;
        assert_eq!(a, BigNum::new());
        a -= &neg_inf;
        assert_eq!(a, pos_inf);
    }

    #[test]
    fn mul() {
        let mut a = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(2_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let c = BigNum {
            sgn: 1,
            cff: BigUint::from(3_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let d = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 3,
            inf: 0,
            nan: false,
        };
        a *= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(2_u32),
                exp: 0,
                inf: 0,
                nan: false,
            }
        );
        a *= &c;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(6_u32),
                exp: 0,
                inf: 0,
                nan: false,
            }
        );
        a *= &d;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(6_u32),
                exp: 3,
                inf: 0,
                nan: false,
            }
        );

        let pos_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: 1,
            nan: false,
        };
        let neg_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: -1,
            nan: false,
        };
        let mut zero = BigNum::new();
        a *= &neg_inf;
        assert_eq!(a, pos_inf);
        a *= &pos_inf;
        assert_eq!(a, pos_inf);
        a *= &pos_inf;
        assert_eq!(a, pos_inf);
        a *= &neg_inf;
        assert_eq!(a, neg_inf);
        a *= &neg_inf;
        assert_eq!(a, pos_inf);
        a *= &c;
        assert_eq!(a, neg_inf);
        a *= &c;
        assert_eq!(a, pos_inf);
        a *= &zero;
        assert_eq!(a, zero);
        zero *= &pos_inf;
        assert_eq!(zero, BigNum::new());
    }

    #[test]
    fn div_assign() {
        let mut a = BigNum {
            sgn: 0,
            cff: BigUint::from(27_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        let b = BigNum {
            sgn: 0,
            cff: BigUint::from(3_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let c = BigNum {
            sgn: 1,
            cff: BigUint::from(3_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let d = BigNum {
            sgn: 1,
            cff: BigUint::from(3_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        let e = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 3,
            inf: 0,
            nan: false,
        };
        a /= &b;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(9_u32),
                exp: 1,
                inf: 0,
                nan: false,
            }
        );
        a /= &c;
        assert_eq!(
            a,
            BigNum {
                sgn: 1,
                cff: BigUint::from(3_u32),
                exp: 1,
                inf: 0,
                nan: false,
            }
        );
        a /= &d;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(1_u32),
                exp: 0,
                inf: 0,
                nan: false,
            }
        );
        a /= &e;
        assert_eq!(
            a,
            BigNum {
                sgn: 0,
                cff: BigUint::from(8_u32),
                exp: 0,
                inf: 0,
                nan: false,
            }
        );

        let pos_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: 1,
            nan: false,
        };
        let neg_inf = BigNum {
            sgn: 0,
            cff: BigUint::new(),
            exp: 0,
            inf: -1,
            nan: false,
        };
        let one = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        let mut zero = BigNum::new();

        a /= &zero;
        assert_eq!(a, pos_inf);
        a /= &zero;
        assert_eq!(a, pos_inf);
        a /= &(-&one);
        assert_eq!(a, neg_inf);
        a /= &pos_inf;
        assert_eq!(a, -&one);
        a /= &pos_inf;
        assert_eq!(a, zero);
        a /= &pos_inf;
        assert_eq!(a, zero);
        a /= &zero;
        assert_eq!(a, BigNum::nan());
        zero /= &pos_inf;
        assert_eq!(zero, BigNum::new());
    }

    #[test]
    fn macro_impl_oper() {
        let n = BigNum {
            sgn: 0,
            cff: BigUint::from(1_u32),
            exp: 1,
            inf: 0,
            nan: false,
        };
        assert_eq!(
            &n + &n + &n,
            BigNum {
                sgn: 0,
                cff: BigUint::from(3_u32),
                exp: 1,
                inf: 0,
                nan: false,
            }
        );
        assert_eq!(
            &n - &n - &n,
            BigNum {
                sgn: 1,
                cff: BigUint::from(1_u32),
                exp: 1,
                inf: 0,
                nan: false,
            }
        );
        assert_eq!(
            &n * &n * &n,
            BigNum {
                sgn: 0,
                cff: BigUint::from(1_u32),
                exp: 3,
                inf: 0,
                nan: false,
            }
        );
        assert_eq!(
            &n / &n / &n,
            BigNum {
                sgn: 0,
                cff: BigUint::from(2_u32),
                exp: 0,
                inf: 0,
                nan: false,
            }
        );

        let deci = BigNum {
            sgn: 0,
            cff: BigUint::from(3_u32),
            exp: 0,
            inf: 0,
            nan: false,
        };
        assert_eq!(
            &n / &n / &deci,
            BigNum {
                sgn: 0,
                cff: BigUint::from(1501199875790165_u64),
                exp: 52,
                inf: 0,
                nan: false,
            }
        );
    }
}
