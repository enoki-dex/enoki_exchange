use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

use candid::{CandidType, Nat};
use num_bigint::BigUint;

use crate::types::{LiquidityAmount, Result, TxError};

#[derive(CandidType, Clone, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StableNat(Vec<u8>);

impl Debug for StableNat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().to_nat().to_string())
    }
}

impl PartialOrd for StableNat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for StableNat {
    fn cmp(&self, other: &Self) -> Ordering {
        let c = self.0.len().cmp(&other.0.len());
        if c == Ordering::Equal {
            for i in 0..self.0.len() {
                let c = self.0[i].cmp(&other.0[i]);
                if c != Ordering::Equal {
                    return c;
                }
            }
            Ordering::Equal
        } else {
            c
        }
    }
}

impl StableNat {
    pub fn is_nonzero(&self) -> bool {
        self.0.iter().any(|&val| val > 0)
    }
    pub fn zero() -> Self {
        Nat::from(0).into()
    }
    pub fn to_nat(self) -> Nat {
        self.into()
    }
    pub fn take_as_nat(&mut self) -> Nat {
        let val = std::mem::take(self);
        val.into()
    }
    pub fn compare_with(&self, value: &Nat) -> Ordering {
        self.clone().to_nat().cmp(value)
    }
    pub fn safe_sub_assign(&mut self, rhs: Self) -> Result<()> {
        let left = std::mem::take(self);
        let result = (left - rhs)?;
        *self = result;
        Ok(())
    }
}

impl From<Nat> for StableNat {
    fn from(v: Nat) -> Self {
        Self(v.0.to_bytes_be())
    }
}

impl From<StableNat> for Nat {
    fn from(v: StableNat) -> Self {
        Self::from(BigUint::from_bytes_be(&v.0))
    }
}

//TODO: make these operations more efficient
impl Add for StableNat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let left = BigUint::from_bytes_be(&self.0);
        let right = BigUint::from_bytes_be(&rhs.0);
        Self((left + right).to_bytes_be())
    }
}

impl Sub for StableNat {
    type Output = Result<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        let left = BigUint::from_bytes_be(&self.0);
        let right = BigUint::from_bytes_be(&rhs.0);
        if left < right {
            Err(TxError::UIntSubtractError.into())
        } else {
            Ok(Self((left - right).to_bytes_be()))
        }
    }
}

impl AddAssign for StableNat {
    fn add_assign(&mut self, rhs: Self) {
        let left = std::mem::take(self);
        *self = left + rhs;
    }
}

impl Mul for StableNat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let left = BigUint::from_bytes_be(&self.0);
        let right = BigUint::from_bytes_be(&rhs.0);
        Self((left * right).to_bytes_be())
    }
}

impl Div for StableNat {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let left = BigUint::from_bytes_be(&self.0);
        let right = BigUint::from_bytes_be(&rhs.0);
        Self((left / right).to_bytes_be())
    }
}

impl Sum for StableNat {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Default::default(), |mut sum, next| {
            sum.add_assign(next);
            sum
        })
    }
}

impl Sum for LiquidityAmount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Default::default(), |mut sum, next| {
            sum.add_assign(next);
            sum
        })
    }
}

#[cfg(test)]
mod stable_nat_tests {
    use std::str::FromStr;

    use candid::Nat;

    use super::*;

    #[test]
    fn test_add() {
        let a: StableNat = Nat::from(45_000_000_000_000u64).into();
        let b: StableNat = Nat::from(30_000_000_000_000u64).into();
        let sum: StableNat = Nat::from(75_000_000_000_000u64).into();
        assert_eq!(sum, a + b);
    }

    #[test]
    fn test_sub() {
        let a: StableNat = Nat::from(45_000_000_000_000u64).into();
        let b: StableNat = Nat::from(30_000_000_000_000u64).into();
        let diff: StableNat = Nat::from(15_000_000_000_000u64).into();
        assert_eq!(diff, (a - b).unwrap());
    }

    #[test]
    fn test_mul() {
        let a: StableNat = Nat::from(45_000_000_000_000u64).into();
        let b: StableNat = Nat::from(3u64).into();
        let prod: StableNat = Nat::from(135_000_000_000_000u64).into();
        assert_eq!(prod, a * b);
    }

    #[test]
    fn test_div() {
        let a: StableNat = Nat::from(45_000_000_000_000u64).into();
        let b: StableNat = Nat::from(3u64).into();
        let q: StableNat = Nat::from(15_000_000_000_000u64).into();
        assert_eq!(q, a / b);
    }

    #[test]
    fn test_ord() {
        let a: StableNat = Nat::from(45_000_000_000_000u64).into();
        let b: StableNat = Nat::from(3u64).into();
        let q: StableNat = Nat::from(15_000_000_000_000u64).into();
        assert!(a > b);
        assert!(a > q);
        assert!(b < q);
        let a: StableNat = Nat::from_str("14_745_697").unwrap().into();
        let b: StableNat = Nat::from_str("29_491_393").unwrap().into();
        println!("a: {:?}, b: {:?}", a.0, b.0);
        assert!(b > a);
    }
}
