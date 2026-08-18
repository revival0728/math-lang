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
struct UintBits(Vec<u64>);

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
}

macro_rules! impl_bit_oper_assgin {
    ($trait:ident, $fname:ident, $oper:tt) => {
        impl $trait for UintBits {
            fn $fname(&mut self, rhs: Self) {
                for (sb, rb) in self.0.iter_mut().zip(rhs.0.iter()) {
                    *sb $oper rb;
                }
                if self.0.len() < rhs.0.len() {
                    self.0.extend_from_slice(&rhs.0[self.0.len()..rhs.0.len()]);
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
        impl $trait for UintBits {
            type Output = UintBits;
            fn $fname(self, rhs: Self) -> Self::Output {
                let mut ret = self.clone();
                ret $oper_assign rhs;
                ret.shrink();
                ret
            }
        }
    };
}
impl_bit_oper!(BitAnd, bitand, &=);
impl_bit_oper!(BitOr, bitor, |=);
impl_bit_oper!(BitXor, bitxor, ^=);

impl Not for UintBits {
    type Output = UintBits;
    fn not(self) -> Self::Output {
        let mut ret = self.clone();
        for bits in ret.0.iter_mut() {
            *bits = !(*bits);
        }
        ret.shrink();
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
        impl Shl<$rhs_type> for UintBits {
            type Output = UintBits;
            fn shl(self, rhs: $rhs_type) -> Self::Output {
                let mut ret = self.clone();
                ret <<= rhs;
                ret
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
        impl Shr<$rhs_type> for UintBits {
            type Output = UintBits;
            fn shr(self, rhs: $rhs_type) -> Self::Output {
                let mut ret = self.clone();
                ret >>= rhs;
                ret
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

    macro_rules! create_bit_oper_assign_test {
        ($name:ident, $oper:tt, $oper_assign:tt) => {
            #[test]
            fn $name() {
                let mut lhs = UintBits::from(vec![1]);
                let rhs = UintBits::from(vec![2, 1]);
                lhs $oper_assign rhs;
                assert_eq!(lhs, UintBits::from(vec![1 $oper 2, 1]));
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
                let ret = lhs $oper rhs;
                assert_eq!(ret, UintBits::from(vec![1 $oper 2, 1]));
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
        assert_eq!(!ubits, UintBits::from(vec![!1, !2]));
    }

    #[test]
    fn shl_assign() {
        let mut ubits = UintBits::new();
        ubits <<= 23;
        assert_eq!(ubits, UintBits::from(vec![0]));
        ubits.0[0] = 1;
        ubits <<= 1;
        assert_eq!(ubits, UintBits::from(vec![2]));
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
        ubits = ubits << 23;
        assert_eq!(ubits, UintBits::from(vec![0]));
        ubits.0[0] = 1;
        ubits = ubits << 1;
        assert_eq!(ubits, UintBits::from(vec![2]));
        ubits = ubits << (64 * 3 + 2);
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
        ubits = ubits >> 23;
        assert_eq!(ubits, UintBits::from(vec![0]));
        ubits.0 = vec![0923];
        ubits = ubits >> 1;
        assert_eq!(ubits, UintBits::from(vec![0923 >> 1]));
        ubits.0 = vec![0, 0, 0, 1234, 3];
        ubits = ubits >> (64 * 3 + 2);
        assert_eq!(ubits, UintBits::from(vec![(1234 >> 2) | (3_u64 << 62)]));
    }
}
