use std::convert::From;
use std::fmt::{Debug, Write};
use std::iter::{DoubleEndedIterator, Iterator};
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

/// Structure provides bit operations for BigUint
///
/// UintBits stores data in little endian.
#[derive(Clone, PartialEq, Eq)]
pub struct UintBits(Vec<u64>);

#[derive(Debug, Clone)]
pub struct BitIter<'a> {
    viter: std::slice::Iter<'a, u64>,
    bits: Option<&'a u64>,
    i: u8,
}

impl Debug for UintBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res: String = self
            .0
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, bits)| {
                if idx == 0 {
                    format!("{:b}", bits)
                } else {
                    format!("{:064b}", bits)
                }
            })
            .collect();

        use std::fmt::Alignment;
        let width = f.width().unwrap_or(0);
        let align = f.align();
        match align {
            Some(Alignment::Left) => write!(f, "{:<w$}", res, w = width),
            Some(Alignment::Right) => write!(f, "{:>w$}", res, w = width),
            Some(Alignment::Center) => write!(f, "{:^w$}", res, w = width),
            None => write!(f, "{}", res),
        }
    }
}

impl UintBits {
    pub fn new() -> Self {
        Self(vec![0])
    }
}

impl Default for UintBits {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<u64>> for UintBits {
    fn from(value: Vec<u64>) -> Self {
        Self(value)
    }
}

impl UintBits {
    pub fn to_le_bytes(&self) -> Vec<u8> {
        const MASK: u64 = (1 << 9) - 1;
        let mut ret = Vec::new();
        for bits in self.0.iter() {
            let mut bits = bits.clone();
            for _ in 0..8 {
                ret.push((bits & MASK) as u8);
                bits >>= 8;
            }
        }
        ret
    }
    pub fn from_le_bytes(bytes: &[u8]) -> Self {
        let mut bits = Self::new();
        for b in (0..bytes.len()).step_by(8) {
            let mut b64 = 0_u64;
            for i in 0..8 {
                let idx = i + b;
                let byte = if idx >= bytes.len() { 0 } else { bytes[idx] } as u64;
                b64 |= byte << (i << 3);
            }
            bits.set_bits(b >> 3, b64);
        }
        bits
    }
}

impl<'a> BitIter<'a> {
    pub fn new(vec: &'a Vec<u64>) -> Self {
        Self {
            viter: vec.iter(),
            bits: None,
            i: 0,
        }
    }
}

impl<'a> Iterator for BitIter<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 64 || self.bits.is_none() {
            self.i = 0;
            self.bits = self.viter.next();
            if let Some(bits) = self.bits {
                let i = self.i;
                self.i += 1;
                Some(((bits >> i) & 1) as u8)
            } else {
                None
            }
        } else {
            let bits = self.bits.unwrap();
            let i = self.i;
            self.i += 1;
            Some(((bits >> i) & 1) as u8)
        }
    }
}

impl<'a> DoubleEndedIterator for BitIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.i == 0 || self.bits.is_none() {
            self.i = 64;
            self.bits = self.viter.next_back();
            if let Some(bits) = self.bits {
                self.i -= 1;
                Some(((bits >> self.i) & 1) as u8)
            } else {
                None
            }
        } else {
            let bits = self.bits.unwrap();
            self.i -= 1;
            Some(((bits >> self.i) & 1) as u8)
        }
    }
}

impl<'a> UintBits {
    pub fn iter(&'a self) -> BitIter<'a> {
        BitIter::new(&self.0)
    }
}

impl UintBits {
    /// pops out redundant leading zeros
    pub fn shrink(&mut self) {
        while self.0.len() > 1 && *self.0.last().unwrap() == 0 {
            self.0.pop();
        }
    }
    pub fn truncate(&mut self, len: usize) {
        self.0
            .truncate((len >> 6) + ((len & ((1 << 6) - 1) != 0) as usize))
    }
    /// return total allocated bit count not actuall bit count
    pub fn len(&self) -> usize {
        self.0.len() << 6
    }
    /// return total bit count
    pub fn bit_len(&self) -> usize {
        self.0.iter().rfold(0, |mut len, bits| {
            if *bits == 0 && len == 0 {
                return len;
            }
            if len == 0 {
                let mut bits = bits.clone();
                let mut sub_len = 0_usize;
                for l in 1..=64 {
                    if bits & 1 == 1 {
                        sub_len = l;
                    }
                    bits >>= 1;
                }
                len += sub_len;
            } else {
                len += 64;
            }
            len
        })
    }
    pub fn get(&self, index: usize) -> u8 {
        let idx = index >> 6;
        let bit = index & ((1 << 6) - 1);
        if idx >= self.0.len() {
            0
        } else {
            ((self.0[idx] >> bit) & 1) as u8
        }
    }
    pub fn set(&mut self, index: usize) {
        let idx = index >> 6;
        let bit = index & ((1 << 6) - 1);
        if self.0.len() <= idx {
            self.0.extend(vec![0; idx - self.0.len() + 1].into_iter());
        }
        self.0[idx] |= 1_u64 << bit;
        self.shrink();
    }
    pub fn reset(&mut self, index: usize) {
        let idx = index >> 6;
        let bit = index & ((1 << 6) - 1);
        if self.0.len() <= idx {
            self.0.extend(vec![0; idx - self.0.len() + 1].into_iter());
        }
        self.0[idx] &= !(1_u64 << bit);
        self.shrink();
    }
    pub fn set_bits(&mut self, data_index: usize, value: u64) {
        if self.0.len() <= data_index {
            self.0
                .extend(vec![0; data_index - self.0.len() + 1].into_iter());
        }
        self.0[data_index] = value;
    }
    /// align most significant side of inner data with bit 0
    /// reutrn aligned length
    /// NOTICE: the align unit is 64-bit not 1-bit
    pub fn align_most(&mut self, other: &Self) -> usize {
        if self.0.len() < other.0.len() {
            let align = other.0.len() - self.0.len();
            self.0.extend(vec![0; align].into_iter());
            align
        } else {
            0
        }
    }
    /// align least significant side of inner data with bit 0
    /// reutrn aligned length
    /// NOTICE: the align unit is 64-bit not 1-bit
    pub fn align_least(&mut self, other: &Self) -> usize {
        if self.0.len() < other.0.len() {
            let align = other.0.len() - self.0.len();
            self.0.reverse();
            self.0.extend(vec![0; align].into_iter());
            self.0.reverse();
            align
        } else {
            0
        }
    }
    pub fn fold_bits<B>(&self, init: B, f: impl FnMut(B, &u64) -> B) -> B {
        self.0.iter().fold(init, f)
    }
    pub fn all_zero(&self) -> bool {
        self.fold_bits(true, |prev, bits| prev & (*bits == 0))
    }
}

macro_rules! impl_bit_oper_assgin {
    ($trait:ident, $fname:ident, $oper:tt) => {
        impl $trait<&Self> for UintBits {
            fn $fname(&mut self, rhs: &Self) {
                self.align_most(rhs);
                for (sb, rb) in self.0.iter_mut().zip(rhs.0.iter()) {
                    *sb $oper rb;
                }
                if self.0.len() > rhs.0.len() {
                    for bits in self.0[rhs.0.len()..].iter_mut() {
                        *bits $oper 0;
                    }
                }
                self.shrink();
            }
        }
    };
}
impl_bit_oper_assgin!(BitAndAssign, bitand_assign, &=);
impl_bit_oper_assgin!(BitOrAssign, bitor_assign, |=);
impl_bit_oper_assgin!(BitXorAssign, bitxor_assign, ^=);

macro_rules! impl_bit_oper {
    ($trait:ident, $fname:ident, $oper_assign:tt) => {
        impl $trait<Self> for &UintBits {
            type Output = UintBits;
            fn $fname(self, rhs: Self) -> Self::Output {
                let mut ret = self.clone();
                ret $oper_assign rhs;
                ret.shrink();
                ret
            }
        }
        impl $trait<&Self> for UintBits {
            type Output = UintBits;
            fn $fname(mut self, rhs: &Self) -> Self::Output {
                self $oper_assign &rhs;
                self.shrink();
                self
            }
        }
    };
}
impl_bit_oper!(BitAnd, bitand, &=);
impl_bit_oper!(BitOr, bitor, |=);
impl_bit_oper!(BitXor, bitxor, ^=);

impl UintBits {
    pub fn not_self(&mut self) {
        for bits in self.0.iter_mut() {
            *bits = !(*bits);
        }
        self.shrink();
    }
}

impl Not for &UintBits {
    type Output = UintBits;
    fn not(self) -> Self::Output {
        let mut ret = self.clone();
        ret.not_self();
        ret
    }
}

macro_rules! impl_shl_assign {
    ($rhs_type:ty $(, $deref:tt)?) => {
        impl ShlAssign<$rhs_type> for UintBits {
            fn shl_assign(&mut self, rhs: $rhs_type) {
                if $($deref)? rhs == 0 {
                    return;
                }

                let step = (rhs >> 6) as usize;
                let shift = rhs & ((1 << 6) - 1);

                // handle step
                if step > 0 {
                    self.0.reverse();
                    self.0.extend(vec![0; step].into_iter());
                    self.0.reverse();
                }

                // handle shift
                if shift == 0 {
                    return;
                }
                let mask = !((1_u64 << (64 - shift)) - 1);
                let mut carry = 0_u64;
                for bits in self.0.iter_mut() {
                    let nxt_carry = (mask & (*bits)) >> (64 - shift);
                    *bits <<= shift;
                    *bits |= carry;
                    carry = nxt_carry;
                }
                if carry != 0 {
                    self.0.push(carry);
                }
                self.shrink();
            }
        }
    };
}
impl_shl_assign!(u8);
impl_shl_assign!(u32);
impl_shl_assign!(u64);
impl_shl_assign!(i8);
impl_shl_assign!(i32);
impl_shl_assign!(i64);
impl_shl_assign!(usize);
impl_shl_assign!(&u8, *);
impl_shl_assign!(&u32, *);
impl_shl_assign!(&u64, *);
impl_shl_assign!(&i8, *);
impl_shl_assign!(&i32, *);
impl_shl_assign!(&i64, *);
impl_shl_assign!(&usize, *);

macro_rules! impl_shl {
    ($rhs_type:ty) => {
        impl Shl<$rhs_type> for &UintBits {
            type Output = UintBits;
            fn shl(self, rhs: $rhs_type) -> Self::Output {
                let mut ret = self.clone();
                ret <<= rhs;
                ret
            }
        }
        impl Shl<$rhs_type> for UintBits {
            type Output = UintBits;
            fn shl(mut self, rhs: $rhs_type) -> Self::Output {
                self <<= rhs;
                self
            }
        }
    };
}
impl_shl!(u8);
impl_shl!(u32);
impl_shl!(u64);
impl_shl!(i8);
impl_shl!(i32);
impl_shl!(i64);
impl_shl!(usize);
impl_shl!(&u8);
impl_shl!(&u32);
impl_shl!(&u64);
impl_shl!(&i8);
impl_shl!(&i32);
impl_shl!(&i64);
impl_shl!(&usize);

macro_rules! impl_shr_assign {
    ($rhs_type:ty $(, $deref:tt)?) => {
        impl ShrAssign<$rhs_type> for UintBits {
            fn shr_assign(&mut self, rhs: $rhs_type) {
                if $($deref)? rhs == 0 {
                    return;
                }

                let step = (rhs >> 6) as usize;
                let shift = rhs & ((1 << 6) - 1);

                // handle step
                if self.0.len() <= step {
                    self.0 = Vec::new();
                    return;
                } else if step > 0 {
                    self.0.reverse();
                    self.0.truncate(self.0.len() - step);
                    self.0.reverse();
                }

                // handle shift
                if shift == 0 {
                    return;
                }
                let mask = (1_u64 << shift) - 1;
                let mut carry = 0_u64;
                for bits in self.0.iter_mut().rev() {
                    let nxt_carry = (mask & (*bits)) << (64 - shift);
                    *bits >>= shift;
                    *bits |= carry;
                    carry = nxt_carry;
                }
                self.shrink();
            }
        }
    };
}
impl_shr_assign!(u8);
impl_shr_assign!(u32);
impl_shr_assign!(u64);
impl_shr_assign!(i8);
impl_shr_assign!(i32);
impl_shr_assign!(i64);
impl_shr_assign!(usize);
impl_shr_assign!(&u8, *);
impl_shr_assign!(&u32, *);
impl_shr_assign!(&u64, *);
impl_shr_assign!(&i8, *);
impl_shr_assign!(&i32, *);
impl_shr_assign!(&i64, *);
impl_shr_assign!(&usize, *);

macro_rules! impl_shr {
    ($rhs_type:ty) => {
        impl Shr<$rhs_type> for &UintBits {
            type Output = UintBits;
            fn shr(self, rhs: $rhs_type) -> Self::Output {
                let mut ret = self.clone();
                ret >>= rhs;
                ret
            }
        }
        impl Shr<$rhs_type> for UintBits {
            type Output = UintBits;
            fn shr(mut self, rhs: $rhs_type) -> Self::Output {
                self >>= rhs;
                self
            }
        }
    };
}
impl_shr!(u8);
impl_shr!(u32);
impl_shr!(u64);
impl_shr!(i8);
impl_shr!(i32);
impl_shr!(i64);
impl_shr!(usize);
impl_shr!(&u8);
impl_shr!(&u32);
impl_shr!(&u64);
impl_shr!(&i8);
impl_shr!(&i32);
impl_shr!(&i64);
impl_shr!(&usize);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn debug() {
        let bits = UintBits::from(vec![1, 1]);
        assert_eq!(
            format!("{:?}", bits),
            "10000000000000000000000000000000000000000000000000000000000000001"
        );
    }

    #[test]
    fn to_le_bytes() {
        let bits = UintBits::from(vec![0, u64::MAX, 0923]);
        let bytes = bits.to_le_bytes();
        let correct: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 155, 3, 0, 0, 0, 0, 0,
            0,
        ];
        assert_eq!(bytes, correct);
    }

    #[test]
    fn from_le_bytes() {
        let b1 = [u8::MAX; 7];
        let b2 = [u8::MAX; 9];
        assert_eq!(
            UintBits::from_le_bytes(&b1),
            UintBits::from(vec![(1 << 56) - 1])
        );
        assert_eq!(
            UintBits::from_le_bytes(&b2),
            UintBits::from(vec![u64::MAX, (1 << 8) - 1])
        );
    }

    #[test]
    fn get() {
        let bits = UintBits::from(vec![0, u64::MAX]);
        assert_eq!(bits.get(0), 0);
        assert_eq!(bits.get(23), 0);
        assert_eq!(bits.get(64), 1);
        assert_eq!(bits.get(120), 1);
    }

    #[test]
    fn set_and_reset() {
        let mut bits = UintBits::new();
        bits.set(0);
        assert_eq!(bits, UintBits::from(vec![1]));
        bits.reset(0);
        assert_eq!(bits, UintBits::from(vec![0]));
        bits.set(64);
        assert_eq!(bits, UintBits::from(vec![0, 1]));
        bits.reset(64);
        assert_eq!(bits, UintBits::from(vec![0]));
    }

    #[test]
    fn align_most() {
        let mut lhs = UintBits::new();
        let rhs = UintBits::from(vec![0, 0]);
        let len = lhs.align_most(&rhs);
        assert_eq!(lhs.0.len(), 2);
        assert_eq!(lhs, rhs);
        assert_eq!(len, 1);
    }

    #[test]
    fn align_least() {
        let mut lhs = UintBits::new();
        let rhs = UintBits::from(vec![0, 0]);
        let len = lhs.align_least(&rhs);
        assert_eq!(lhs.0.len(), 2);
        assert_eq!(lhs, rhs);
        assert_eq!(len, 1);
    }

    #[test]
    fn bit_len() {
        let bits = UintBits::from(vec![0923, 1, 0]);
        assert_eq!(bits.bit_len(), 65);
    }

    #[test]
    fn len_and_truncate() {
        let mut bits = UintBits::from(vec![1; 4]);
        assert_eq!(bits.len(), 4 * 64);
        bits.truncate(65);
        assert_eq!(bits.len(), 2 * 64);
        bits.truncate(64);
        assert_eq!(bits.len(), 1 * 64);
        bits.truncate(1);
        assert_eq!(bits.len(), 1 * 64);
    }

    #[test]
    fn iter() {
        let bits = UintBits::from(vec![0, u64::MAX]);
        let bit_arr = bits.iter().collect::<Vec<u8>>();
        let mut correct = Vec::new();
        correct.extend(vec![0_u8; 64].into_iter());
        correct.extend(vec![1_u8; 64].into_iter());
        assert_eq!(bit_arr, correct);
    }

    #[test]
    fn double_ended_iter() {
        let bits = UintBits::from(vec![0, u64::MAX]);
        let bit_arr = bits.iter().rev().collect::<Vec<u8>>();
        let mut correct = Vec::new();
        correct.extend(vec![1_u8; 64].into_iter());
        correct.extend(vec![0_u8; 64].into_iter());
        assert_eq!(bit_arr, correct);
    }

    macro_rules! create_bit_oper_assign_test {
        ($name:ident, $oper:tt, $oper_assign:tt) => {
            #[test]
            fn $name() {
                let mut lhs = UintBits::from(vec![1]);
                let rhs = UintBits::from(vec![2, 1]);
                lhs $oper_assign &rhs;
                let mut ans = UintBits::from(vec![1 $oper 2, 1 $oper 0]);
                ans.shrink();
                assert_eq!(lhs, ans);
            }
        };
    }
    create_bit_oper_assign_test!(bitand_assign, &, &=);
    create_bit_oper_assign_test!(bitor_assign, |, |=);
    create_bit_oper_assign_test!(bitxor_assign, ^, ^=);

    macro_rules! creat_bit_oper_test {
        ($name:ident, $oper:tt) => {
            #[test]
            fn $name() {
                let lhs = UintBits::from(vec![1]);
                let rhs = UintBits::from(vec![2, 1]);
                let ret = (&lhs) $oper (&rhs);
                let mut ans = UintBits::from(vec![1 $oper 2, 1 $oper 0]);
                ans.shrink();
                assert_eq!(ret, ans);
            }
        };
    }
    creat_bit_oper_test!(bitand, &);
    creat_bit_oper_test!(bitor, |);
    creat_bit_oper_test!(bitxor, ^);

    #[test]
    fn not() {
        let inner = vec![1, 2];
        let ubits = UintBits::from(inner.clone());
        assert_eq!(ubits, UintBits::from(inner));
        assert_eq!(!(&ubits), UintBits::from(vec![!1, !2]));
    }

    #[test]
    fn shl_assign() {
        let mut ubits = UintBits::new();
        ubits <<= 23;
        assert_eq!(ubits, UintBits::from(vec![0]));
        ubits.0[0] = 1;
        ubits <<= 1;
        assert_eq!(ubits, UintBits::from(vec![2]));
        ubits.0 = vec![u64::MAX];
        ubits <<= 1;
        assert_eq!(ubits, UintBits::from(vec![u64::MAX - 1, 1]));
        ubits.0 = vec![1234, u64::MAX, 5678];
        ubits <<= 64 * 3 + 2;
        assert_eq!(
            ubits,
            UintBits::from(vec![0, 0, 0, 1234 << 2, u64::MAX - 3, (5678 << 2) | 3])
        );
    }

    #[test]
    fn shl() {
        let mut ubits = UintBits::new();
        ubits = (&ubits) << 23;
        assert_eq!(ubits, UintBits::from(vec![0]));
        ubits.0[0] = 1;
        ubits = (&ubits) << 1;
        assert_eq!(ubits, UintBits::from(vec![2]));
        ubits = (&ubits) << (64 * 3 + 2);
        assert_eq!(ubits, UintBits::from(vec![0, 0, 0, 2 << 2]));
    }

    #[test]
    fn shr_assign() {
        let mut ubits = UintBits::new();
        ubits >>= 23;
        assert_eq!(ubits, UintBits::from(vec![0]));
        ubits.0 = vec![0923];
        ubits >>= 1;
        assert_eq!(ubits, UintBits::from(vec![0923 >> 1]));
        ubits.0 = vec![0, 0, 0, 1234, 3];
        ubits >>= 64 * 3 + 2;
        assert_eq!(ubits, UintBits::from(vec![(1234 >> 2) | (3_u64 << 62)]));
    }

    #[test]
    fn shr() {
        let mut ubits = UintBits::new();
        ubits = (&ubits) >> 23;
        assert_eq!(ubits, UintBits::from(vec![0]));
        ubits.0 = vec![0923];
        ubits = (&ubits) >> 1;
        assert_eq!(ubits, UintBits::from(vec![0923 >> 1]));
        ubits.0 = vec![0, 0, 0, 1234, 3];
        ubits = (&ubits) >> (64 * 3 + 2);
        assert_eq!(ubits, UintBits::from(vec![(1234 >> 2) | (3_u64 << 62)]));
    }
}
