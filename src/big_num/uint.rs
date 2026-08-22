use super::decimal::Decimal;
use super::ubits::UintBits;
use std::cmp::Ord;
use std::convert::From;
use std::fmt::Display;
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

impl BigUint {
    pub fn is_zero(&self) -> bool {
        self.bits.all_zero()
    }
    pub fn bit_count(&self) -> usize {
        self.bits.bit_len()
    }
    pub fn bit_capacity(&self) -> usize {
        self.bits.len()
    }
    pub fn into_bits(self) -> UintBits {
        self.bits
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
impl_from!(u128);

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
    pub fn div_rem_self(&mut self, rhs: &Self) -> Self {
        let sbl = self.bits.bit_len();
        let rbl = rhs.bits.bit_len();
        if sbl < rbl {
            return Self::new();
        }
        let mut rhs = rhs.clone();
        let qlen = sbl - rbl;
        rhs.bits <<= qlen;
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

impl BigUint {
    /// calcuates decimal part of division, precision provides as 10-base
    pub fn div_decimal(&self, rhs: &Self, precision: u8) -> Decimal {
        const TLOG2: u32 = 10_u32.ilog2();
        const MAGIC: u32 = 32192; // decimal part of log2(10)
        const SHIFT: u8 = 16;
        let mut lhs = self.clone();
        let lbl = lhs.bits.bit_len();
        let rbl = rhs.bits.bit_len();
        if lbl > rbl {
            return Decimal::new(UintBits::new(), 0, 0);
        }
        let p = precision as u32;
        let p2b = p * TLOG2 + ((p * MAGIC) >> SHIFT);
        let base = (rbl - lbl) as u32 + p2b;
        lhs.bits <<= base;
        let q = &lhs / &rhs;
        Decimal::new(q.bits, base, precision)
    }
}

impl Display for BigUint {
    /// the double dabble algorithm
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = self.bits.clone();
        let bit_len = buf.bit_len();
        if bit_len <= 3 {
            let bytes = self.bits.to_le_bytes();
            return write!(f, "{}", if bytes.is_empty() { 0 } else { bytes[0] });
        }
        buf <<= 3;
        let mut dlen = 0_usize;
        for l in 3..bit_len {
            let mut unit = 0_usize;
            let len = l + dlen;
            while unit <= len {
                let mut tmp = buf.get(unit + bit_len)
                    | (buf.get(unit + bit_len + 1) << 1)
                    | (buf.get(unit + bit_len + 2) << 2)
                    | (buf.get(unit + bit_len + 3) << 3);
                if tmp >= 5 {
                    tmp += 3;
                }
                for i in 0..4 {
                    let index = unit + bit_len + i;
                    if tmp & 1 == 1 {
                        buf.set(index);
                    } else {
                        buf.reset(index);
                    }
                    tmp >>= 1;
                }
                unit += 4;
            }
            const MASK_2: usize = !((1 << 2) - 1);
            // check if most bit of current leading 10-base 4 bits is 1
            if buf.get(bit_len + (len & MASK_2) + 4 * ((len & !MASK_2) != 0) as usize) == 1 {
                dlen += 1;
            }
            buf <<= 1;
        }
        buf >>= bit_len;
        let bytes = buf.to_le_bytes();
        const MASK_4: u8 = (1 << 4) - 1;
        let res: String = bytes.into_iter().rfold(String::new(), |mut res, bytes| {
            let f = bytes >> 4;
            let s = bytes & MASK_4;
            if f != 0 || !res.is_empty() {
                res.extend(f.to_string().chars());
            }
            if s != 0 || !res.is_empty() {
                res.extend(s.to_string().chars());
            }
            res
        });
        write!(f, "{}", res)
    }
}

impl From<&str> for BigUint {
    fn from(value: &str) -> Self {
        const ASCII_ZERO: u8 = '0' as u8;
        let bytes = {
            let mut bytes = Vec::new();
            let ascii = value.as_bytes();
            for b in (0..ascii.len()).rev().step_by(2) {
                let f = ascii[b] - ASCII_ZERO;
                let s = if b < 1 { ASCII_ZERO } else { ascii[b - 1] } - ASCII_ZERO;
                bytes.push((s << 4) | f);
            }
            bytes
        };
        let bit_len = bytes.len() << 3;
        let mut buf = UintBits::from_le_bytes(&bytes);
        buf <<= bit_len;
        for _ in 0..bit_len {
            buf >>= 1;
            let mut unit = 0_usize;
            while unit <= bit_len {
                let mut tmp = buf.get(unit + bit_len)
                    | (buf.get(unit + bit_len + 1) << 1)
                    | (buf.get(unit + bit_len + 2) << 2)
                    | (buf.get(unit + bit_len + 3) << 3);
                if tmp >= 8 {
                    tmp -= 3;
                }
                for i in 0..4 {
                    let index = unit + bit_len + i;
                    if tmp & 1 == 1 {
                        buf.set(index);
                    } else {
                        buf.reset(index);
                    }
                    tmp >>= 1;
                }
                unit += 4;
            }
        }
        buf.shrink();
        BigUint::from(buf)
    }
}

impl From<String> for BigUint {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
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
        let mut lg1 = BigUint::from("378213101214913900000000000000000000");
        let lg2 = BigUint::from("9223372036854775808");
        let mut lg3 = BigUint::from("381371410412680000000000000000");
        let lg4 = BigUint::from("9007199254740992");
        a /= &b;
        a /= &c;
        lg1 /= &lg2;
        lg3 /= &lg4;
        assert_eq!(a, BigUint::from(1_u32));
        assert_eq!(lg1, BigUint::from("41005946599968962"));
        assert_eq!(lg3, BigUint::from("42340732077392"));
    }

    #[test]
    fn div() {
        let a = BigUint::from(17_u32);
        let b = BigUint::from(4_u32);
        let c = BigUint::from(3_u32);
        let lg1 = BigUint::from("378213101214913900000000000000000000");
        let lg2 = BigUint::from("9223372036854775808");
        assert_eq!(&a / &b / &c, BigUint::from(1_u32));
        assert_eq!(&lg1 / &lg2, BigUint::from("41005946599968962"))
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

    #[test]
    fn div_decimal() {
        let a = BigUint::from(1_u32);
        let b = BigUint::from(2_u32);
        let c = BigUint::from(7_u32);
        assert_eq!(
            a.div_decimal(&b, 2),
            Decimal::new(UintBits::from(vec![64]), 7, 2)
        );
        assert_eq!(
            a.div_decimal(&c, 7),
            Decimal::new(UintBits::from(vec![9586980]), 26, 7)
        );
    }

    #[test]
    fn display_to_string() {
        let a = BigUint::from(0_u32);
        let b = BigUint::from(1_u32);
        let c = BigUint::from(123456_u32);
        let d = BigUint::from(102304055008_u64);
        let large = BigUint::from(UintBits::from(vec![1, 1]));
        assert_eq!(a.to_string(), "0");
        assert_eq!(b.to_string(), "1");
        assert_eq!(c.to_string(), "123456");
        assert_eq!(d.to_string(), "102304055008");
        assert_eq!(large.to_string(), "18446744073709551617");
    }

    #[test]
    fn from_and_to_string() {
        let a = "0";
        let b = "1";
        let c = "123456";
        let d = "102304055008";
        let large = "18446744073709551617";
        let lg1 = "378213101214913900000000000000000000";
        let lg2 = "9223372036854775808";
        assert_eq!(BigUint::from(a).to_string(), a);
        assert_eq!(BigUint::from(b).to_string(), b);
        assert_eq!(BigUint::from(c).to_string(), c);
        assert_eq!(BigUint::from(d).to_string(), d);
        assert_eq!(BigUint::from(large).to_string(), large);
        assert_eq!(BigUint::from(lg1).to_string(), lg1);
        assert_eq!(BigUint::from(lg2).to_string(), lg2);
    }
}
