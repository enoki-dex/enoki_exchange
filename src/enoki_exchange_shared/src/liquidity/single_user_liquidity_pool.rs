use std::ops::{AddAssign, SubAssign};

use candid::CandidType;

use crate::types::*;

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct SingleUserLiquidityPool {
    liquidity: LiquidityAmount,
    pending_add: Vec<TokenAmount>,
    pending_remove: Vec<TokenAmount>,
    pending_add_locked: Vec<TokenAmount>,
    pending_remove_locked: Vec<TokenAmount>,
}

impl SingleUserLiquidityPool {
    pub fn get_liquidity(&self) -> &LiquidityAmount {
        &self.liquidity
    }
    pub fn nothing_pending(&self) -> bool {
        self.pending_add.is_empty() && self.pending_remove.is_empty()
    }
    pub fn user_add_liquidity(&mut self, amount: TokenAmount) {
        if amount.amount.is_nonzero() {
            self.pending_add.push(amount);
        }
    }
    pub fn user_remove_liquidity(&mut self, amount: LiquidityAmount) -> Result<()> {
        let existing = &self.liquidity;
        let amount_a = amount.token_a.min(existing.token_a.clone());
        let amount_b = amount.token_b.min(existing.token_b.clone());
        if amount_a.is_nonzero() {
            self.pending_remove.push(TokenAmount {
                token: EnokiToken::TokenA,
                amount: amount_a,
            });
        }
        if amount_b.is_nonzero() {
            self.pending_remove.push(TokenAmount {
                token: EnokiToken::TokenB,
                amount: amount_b,
            });
        }
        Ok(())
    }
    pub fn lock_liquidity(&mut self) -> (LiquidityAmount, LiquidityAmount) {
        let to_add = std::mem::take(&mut self.pending_add);
        self.pending_add_locked.extend(to_add.clone());
        let to_remove = std::mem::take(&mut self.pending_remove);
        self.pending_remove_locked.extend(to_remove.clone());
        (
            to_add.into_iter().collect(),
            to_remove.into_iter().collect(),
        )
    }
    pub fn get_locked_add_item(&mut self, index: usize) -> Option<&mut TokenAmount> {
        if index < self.pending_add_locked.len() {
            Some(&mut self.pending_add_locked[index])
        } else {
            None
        }
    }
    pub fn get_locked_remove_item(&mut self, index: usize) -> Option<&mut TokenAmount> {
        if index < self.pending_remove_locked.len() {
            Some(&mut self.pending_remove_locked[index])
        } else {
            None
        }
    }
    pub fn get_liquidity_mut(&mut self, token: &EnokiToken) -> &mut StableNat {
        self.liquidity.get_mut(token)
    }
    pub fn remove_zeros(&mut self) {
        self.pending_add_locked
            .retain(|amount| amount.amount.is_nonzero());
        self.pending_remove_locked
            .retain(|amount| amount.amount.is_nonzero());
    }
    pub fn apply_traded(&mut self, traded: &LiquidityTrades) {
        self.liquidity.add_assign(traded.increased.clone());
        self.liquidity.sub_assign(traded.decreased.clone());
    }
}
