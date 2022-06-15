use std::ops::{AddAssign, Div, Sub};

use crate::types::*;

impl FromIterator<TokenAmount> for LiquidityAmount {
    fn from_iter<T: IntoIterator<Item=TokenAmount>>(iter: T) -> Self {
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
            token_a: self.token_a.to_nat().div(val).into(),
            token_b: self.token_b.to_nat().div(val).into(),
        }
    }
    pub fn sub_assign_or_zero(&mut self, other: Self) {
        if self.token_a > other.token_a {
            self.token_a.safe_sub_assign(other.token_a).unwrap();
        } else {
            self.token_a = StableNat::zero();
        }
        if self.token_b > other.token_b {
            self.token_b.safe_sub_assign(other.token_b).unwrap();
        } else {
            self.token_b = StableNat::zero();
        }
    }
    pub fn sub_or_zero(&self, other: &Self) -> Self {
        Self {
            token_a: if self.token_a > other.token_a {
                self.token_a.clone().sub(other.token_a.clone()).unwrap()
            } else {
                StableNat::zero()
            },
            token_b: if self.token_b > other.token_b {
                self.token_b.clone().sub(other.token_b.clone()).unwrap()
            } else {
                StableNat::zero()
            },
        }
    }
    pub fn safe_sub_assign(&mut self, rhs: Self) -> Result<()> {
        self.token_a.safe_sub_assign(rhs.token_a)?;
        self.token_b.safe_sub_assign(rhs.token_b)?;
        Ok(())
    }
}

impl AddAssign for LiquidityAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.token_a.add_assign(rhs.token_a);
        self.token_b.add_assign(rhs.token_b);
    }
}

impl LiquidityTrades {
    pub fn safe_sub_assign(&mut self, rhs: Self) -> Result<()> {
        self.increased.safe_sub_assign(rhs.increased)?;
        self.decreased.safe_sub_assign(rhs.decreased)?;
        Ok(())
    }
}

impl AddAssign for LiquidityTrades {
    fn add_assign(&mut self, rhs: Self) {
        self.increased.add_assign(rhs.increased);
        self.decreased.add_assign(rhs.decreased);
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

impl From<LiquidityAmount> for LiquidityAmountNat {
    fn from(val: LiquidityAmount) -> Self {
        Self {
            token_a: val.token_a.into(),
            token_b: val.token_b.into(),
        }
    }
}

impl EnokiToken {
    pub fn opposite(&self) -> Self {
        match self {
            EnokiToken::TokenA => EnokiToken::TokenB,
            EnokiToken::TokenB => EnokiToken::TokenA,
        }
    }
}