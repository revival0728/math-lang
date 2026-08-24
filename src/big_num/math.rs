// use super::num::BigNum;

// const E: f64 = std::f64::consts::E;

// pub fn sqrt(x: &BigNum) -> BigNum {
//     const N: u32 = 10;
//     let mut res = BigNum::from(1_u32);
//     let two = BigNum::from(2_u32);
//     for _ in 0..N {
//         res = (&res + &(x / &res)) / &two;
//     }
//     res
// }

// #[cfg(test)]
// mod test {
//     use super::BigNum;

//     #[test]
//     fn sqrt() {
//         let n = 68127_u32;
//         let x = BigNum::from(n);
//         assert_eq!(
//             super::sqrt(&x).to_float_str(15),
//             (n as f64).sqrt().to_string()[0..17]
//         );
//     }
// }
