use std::iter::Sum;
use std::ops::{AddAssign, Div, Mul, SubAssign};

use lazy_static::lazy_static;

use crate::types::*;

impl From<Nat> for StableNat {
    fn from(v: Nat) -> Self {
        Self(v)
    }
}

impl From<StableNat> for Nat {
    fn from(v: StableNat) -> Self {
        v.0
    }
}

impl FromIterator<TokenAmount> for LiquidityAmount {
    fn from_iter<T: IntoIterator<Item = TokenAmount>>(iter: T) -> Self {
        let mut result = Self::default();
        for item in iter {
            match item.token {
                EnokiToken::TokenA => {
                    result.token_a += item.amount;
                }
                EnokiToken::TokenB => {
                    result.token_b += item.amount;
                }
            }
        }
        result
    }
}

impl LiquidityAmount {
    pub fn get(&self, token: &EnokiToken) -> &StableNat {
        match token {
            EnokiToken::TokenA => &self.token_a,
            EnokiToken::TokenB => &self.token_b,
        }
    }
    pub fn get_mut(&mut self, token: &EnokiToken) -> &mut StableNat {
        match token {
            EnokiToken::TokenA => &mut self.token_a,
            EnokiToken::TokenB => &mut self.token_b,
        }
    }
}

lazy_static! {
    static ref ZERO: Nat = Nat::from(0);
}

impl StableNat {
    pub fn min(&self, other: &Self) -> Self {
        Self(self.0.clone().min(other.0.clone()))
    }
    pub fn is_nonzero(&self) -> bool {
        self.0 > *ZERO
    }
}

impl AddAssign for StableNat {
    fn add_assign(&mut self, rhs: Self) {
        let lhs = std::mem::take(&mut self.0);
        *self = Self(lhs + rhs.0)
    }
}

impl SubAssign for StableNat {
    fn sub_assign(&mut self, rhs: Self) {
        let lhs = std::mem::take(&mut self.0);
        *self = Self(lhs - rhs.0)
    }
}

impl Mul for StableNat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for StableNat {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Sum for StableNat {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(Default::default(), |mut sum, next| {
            sum.add_assign(next);
            sum
        })
    }
}

impl AddAssign for LiquidityAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.token_a.add_assign(rhs.token_a);
        self.token_b.add_assign(rhs.token_b);
    }
}