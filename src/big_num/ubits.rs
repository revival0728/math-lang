use std::convert::From;
use std::fmt::Display;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

/// Structure provides bit operations for Big_Uint
///
/// UintBits stores data in little endian.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UintBits(Vec<u64>);

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
    /// pops out redundant leading zeros
    fn shrink(&mut self) {
        while self.0.len() > 1 && *self.0.last().unwrap() == 0 {
            self.0.pop();
        }
    }
    pub fn len(&self) -> usize {
        self.0.len() * 64
    }
    pub fn set(&mut self, index: usize) {
        let idx = index / 64;
        let bit = index % 64;
        if self.0.len() <= idx {
            self.0.extend(vec![0; idx - self.0.len() + 1].into_iter());
        }
        self.0[idx] |= 1_u64 << bit;
        self.shrink();
    }
    pub fn reset(&mut self, index: usize) {
        let idx = index / 64;
        let bit = index % 64;
        if self.0.len() <= idx {
            self.0.extend(vec![0; idx - self.0.len() + 1].into_iter());
        }
        self.0[idx] &= !(1_u64 << bit);
        self.shrink();
    }
    pub fn set_bits(&mut self, index: usize, value: u64) {
        self.0[index] = value;
    }
    /// align inner data bits with 0
    pub fn align(&mut self, other: &Self) {
        if self.0.len() < other.0.len() {
            self.0
                .extend(vec![0; other.0.len() - self.0.len()].into_iter());
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
                self.align(rhs);
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
        impl $trait<UintBits> for &UintBits {
            type Output = UintBits;
            fn $fname(self, rhs: UintBits) -> Self::Output {
                let mut ret = self.clone();
                ret $oper_assign &rhs;
                ret.shrink();
                ret
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

                let step = (rhs / 64) as usize;
                let shift = rhs % 64;

                // handle step
                self.0.reverse();
                self.0.extend(vec![0; step].into_iter());
                self.0.reverse();

                // handle shift
                assert!(shift < 64);
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

                let step = (rhs / 64) as usize;
                let shift = rhs % 64;

                // handle step
                if self.0.len() <= step {
                    self.0 = Vec::new();
                    return;
                } else {
                    self.0.reverse();
                    self.0.truncate(self.0.len() - step);
                    self.0.reverse();
                }

                // handle shift
                assert!(shift < 64);
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
    fn align() {
        let mut lhs = UintBits::new();
        let rhs = UintBits::from(vec![0, 0]);
        lhs.align(&rhs);
        assert_eq!(lhs.0.len(), 2);
        assert_eq!(lhs, rhs);
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
