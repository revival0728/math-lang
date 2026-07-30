use std::fmt::Display;
use std::ops::{Add, Mul, Sub};

const UINT_BASE: u64 = (u32::MAX as u64) + 1;

// BigUInt = sum(data[i] * UINT_BASE ^ i)
#[derive(Clone, Debug)]
pub struct BigUInt {
    data: Vec<u32>,
}

impl BigUInt {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl Add for &BigUInt {
    type Output = BigUInt;
    fn add(self, rhs: Self) -> Self::Output {
        let max_size = self.data.len().max(rhs.data.len()) + 1;
        let mut data = Vec::with_capacity(max_size);
        let mut carry = 0_u64;
        for i in 0..max_size {
            let l = self.data[i] as u64;
            let r = rhs.data[i] as u64;
            let mut val = l + r + carry;
            carry = 0;
            while val >= UINT_BASE {
                val -= UINT_BASE;
                carry += 1;
            }
            data.push(val as u32);
        }
        Self::Output { data }
    }
}

// shift right and return bit
impl BigUInt {
    fn shl_return(&mut self) -> u8 {
        let mut last: u32 = 0;
        for num in self.data.iter_mut() {
            let new_last = *num >> (u32::BITS - 1);
            *num <<= 1;
            *num |= last;
            last = new_last;
        }
        last as u8
    }
}

// double dabble algorithm
impl Display for BigUInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const HFB_MASK: u8 = (1 << 4) - 1;
        let mut num = self.clone();
        let total_bytes = num.data.len() << 2;
        let mut deci: Vec<u8> = vec![0; total_bytes];
        for _ in 0..(total_bytes << 3) {
            let mut bit = num.shl_return();
            for n in deci.iter_mut() {
                let new_bit = *n >> (u8::BITS - 1);
                *n <<= 1;
                *n |= bit;
                bit = new_bit;
            }
            for n in deci.iter_mut() {
                let mut fh = *n & HFB_MASK;
                let mut sh = *n >> 4;
                if fh >= 5 {
                    fh = (fh + 3) & HFB_MASK; // simulates half byte overflow
                }
                if sh >= 5 {
                    sh = (sh + 3) & HFB_MASK;
                }
                *n = (sh << 4) | fh;
            }
            dbg!(&deci);
        }
        let s: Vec<String> = deci
            .into_iter()
            .map(|byte| {
                let f = char::from_digit((byte & HFB_MASK) as u32, 10).unwrap();
                let s = char::from_digit((byte >> 4) as u32, 10).unwrap();
                format!("{}{}", f, s)
            })
            .collect();
        let s: String = s.join("").chars().rev().collect();
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod test {
    use crate::big_uint::BigUInt;

    #[test]
    fn shr_return() {
        let mut num1 = BigUInt::new();
        num1.data.push(12345);

        let correct: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            0, 0, 1,
        ];
        let answer: Vec<u8> = (0..correct.len()).map(|_| num1.shl_return()).collect();
        assert_eq!(answer, correct);
    }

    #[test]
    fn display() {
        let mut num1 = BigUInt::new();
        num1.data.push(12345);

        assert_eq!(num1.to_string(), "12345");
    }
}
