use std::ops::{AddAssign, SubAssign};

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
