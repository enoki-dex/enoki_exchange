use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

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
    pub fn div_int(self, val: usize) -> Self {
        Self {
            token_a: StableNat(self.token_a.0.div(val)),
            token_b: StableNat(self.token_b.0.div(val)),
        }
    }
    pub fn sub_assign_or_zero(&mut self, other: Self) {
        if self.token_a > other.token_a {
            self.token_a.sub_assign(other.token_a);
        } else {
            self.token_a = StableNat::zero();
        }
        if self.token_b > other.token_b {
            self.token_b.sub_assign(other.token_b);
        } else {
            self.token_b = StableNat::zero();
        }
    }
    pub fn sub_or_zero(&self, other: &Self) -> Self {
        Self {
            token_a: if self.token_a > other.token_a {
                self.token_a.clone().sub(other.token_a.clone())
            } else {
                StableNat::zero()
            },
            token_b: if self.token_b > other.token_b {
                self.token_b.clone().sub(other.token_b.clone())
            } else {
                StableNat::zero()
            },
        }
    }
}

lazy_static! {
    static ref ZERO: Nat = Nat::from(0);
}

impl StableNat {
    pub fn is_nonzero(&self) -> bool {
        self.0 > *ZERO
    }
    pub fn zero() -> Self {
        Self(Nat::from(0))
    }
}

impl Add for StableNat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for StableNat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
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
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
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

impl SubAssign for LiquidityAmount {
    fn sub_assign(&mut self, rhs: Self) {
        self.token_a.sub_assign(rhs.token_a);
        self.token_b.sub_assign(rhs.token_b);
    }
}

impl AddAssign for LiquidityTrades {
    fn add_assign(&mut self, rhs: Self) {
        self.increased.add_assign(rhs.increased);
        self.decreased.add_assign(rhs.decreased);
    }
}

impl SubAssign for LiquidityTrades {
    fn sub_assign(&mut self, rhs: Self) {
        self.increased.sub_assign(rhs.increased);
        self.decreased.sub_assign(rhs.decreased);
    }
}

impl From<OrderInfo> for Order {
    fn from(info: OrderInfo) -> Self {
        Self {
            state: OrderState {
                status: OrderStatus::Pending,
                quantity_remaining: info.quantity.clone(),
                marker_makers: vec![],
            },
            info,
        }
    }
}

impl AggregateBidAsk {
    pub fn change_to_next(&mut self, next: &Self) {
        let mut next = next.clone();
        //for security/extreme arbitrage reasons (waiting for brokers to be synchronized), bid/ask cannot intersect between rounds
        let last_bid = self.bids.keys().last();
        let last_ask = self.asks.keys().next();
        if let Some(&last_bid) = last_bid {
            while let Some(&ask) = next.asks.keys().next() {
                if ask < last_bid {
                    next.asks.remove(&ask);
                } else {
                    break;
                }
            }
        }
        if let Some(&last_ask) = last_ask {
            while let Some(&bid) = next.bids.keys().last() {
                if bid > last_ask {
                    next.bids.remove(&bid);
                } else {
                    break;
                }
            }
        }
        *self = next;
    }
}
