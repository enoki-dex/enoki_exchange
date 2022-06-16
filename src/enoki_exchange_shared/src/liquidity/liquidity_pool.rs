use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::AddAssign;
use std::str::FromStr;

use candid::{CandidType, Nat, Principal};
use lazy_static::lazy_static;

use crate::types::*;

lazy_static! {
    //TODO: make it based on fee
    static ref MIN_AMOUNT_TO_WITHDRAW: StableNat = Nat::from_str("30_000_000").unwrap().into();
}

#[derive(serde::Serialize, serde::Deserialize, CandidType, Clone, Debug, Default)]
pub struct LiquidityPool {
    liquidity: HashMap<Principal, LiquidityAmount>,
    pending_add: Vec<(Principal, TokenAmount)>,
    pending_remove: Vec<(Principal, TokenAmount)>,
    pending_add_locked: Vec<(Principal, TokenAmount)>,
    pending_remove_locked: Vec<(Principal, TokenAmount)>,
    user_net_deposits: HashMap<Principal, LiquidityTrades>,
}

pub struct LiquidityPoolTotalBalance<'a>(&'a LiquidityPool);

impl<'a> LiquidityPoolTotalBalance<'a> {
    pub fn get_total_balances(&self) -> LiquidityAmount {
        self.0.liquidity.iter().map(|(_, val)| val.clone()).sum()
    }
    pub fn new(pool: &'a LiquidityPool) -> Self {
        Self(pool)
    }
}

impl<'a> Debug for LiquidityPoolTotalBalance<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get_total_balances())
    }
}

impl LiquidityPool {
    pub fn get_user_liquidity(&self, user: Principal) -> Option<LiquidityAmount> {
        self.liquidity.get(&user).cloned()
    }
    pub fn get_user_net_deposits(&self, user: Principal) -> Option<LiquidityTrades> {
        self.user_net_deposits.get(&user).cloned()
    }
    pub fn nothing_pending(&self) -> bool {
        self.pending_add.is_empty() && self.pending_remove.is_empty()
    }
    pub fn user_add_liquidity(&mut self, user: Principal, amount: TokenAmount) {
        if amount.amount.is_nonzero() {
            ic_cdk::println!(
                "[worker] increased user {} pending liquidity by {:?}",
                user,
                amount
            );
            self.pending_add.push((user, amount));
        }
    }
    pub fn user_remove_liquidity(
        &mut self,
        user: Principal,
        amount: LiquidityAmount,
    ) -> Result<()> {
        let existing = self
            .liquidity
            .get(&user)
            .ok_or(TxError::UserNotRegistered {
                user: user.to_string(),
                registry: ic_cdk::id().to_string(),
            })?;
        let amount_a = amount.token_a.min(existing.token_a.clone());
        let amount_b = amount.token_b.min(existing.token_b.clone());
        if amount_a >= *MIN_AMOUNT_TO_WITHDRAW {
            ic_cdk::println!(
                "[worker] decreased user {} pending liquidity A by {:?}",
                user,
                amount_a
            );
            self.pending_remove.push((
                user,
                TokenAmount {
                    token: EnokiToken::TokenA,
                    amount: amount_a,
                },
            ));
        }
        if amount_b >= *MIN_AMOUNT_TO_WITHDRAW {
            ic_cdk::println!(
                "[worker] decreased user {} pending liquidity B by {:?}",
                user,
                amount_b
            );
            self.pending_remove.push((
                user,
                TokenAmount {
                    token: EnokiToken::TokenB,
                    amount: amount_b,
                },
            ));
        }
        Ok(())
    }
    pub fn lock_liquidity(&mut self) -> (LiquidityAmount, LiquidityAmount) {
        ic_cdk::println!(
            "[worker] locking {} pending add and {} pending remove",
            self.pending_add.len(),
            self.pending_remove.len()
        );
        let to_add = std::mem::take(&mut self.pending_add);
        let to_remove = std::mem::take(&mut self.pending_remove);
        self.pending_add_locked.extend(to_add.clone());
        self.pending_remove_locked.extend(to_remove.clone());
        (
            to_add.into_iter().map(|(_, i)| i).collect(),
            to_remove.into_iter().map(|(_, i)| i).collect(),
        )
    }
    fn consolidate_liquidity_by_principal(
        liquidity: &[(Principal, TokenAmount)],
    ) -> HashMap<Principal, LiquidityAmount> {
        liquidity
            .iter()
            .fold(HashMap::new(), |mut total, (principal, amount)| {
                total
                    .entry(*principal)
                    .or_default()
                    .get_mut(&amount.token)
                    .add_assign(amount.amount.clone());
                total
            })
    }
    pub fn count_locked_add_liquidity_by_principal(&self) -> HashMap<Principal, LiquidityAmount> {
        Self::consolidate_liquidity_by_principal(&self.pending_add_locked)
    }
    pub fn count_locked_remove_liquidity_by_principal(
        &self,
    ) -> HashMap<Principal, LiquidityAmount> {
        Self::consolidate_liquidity_by_principal(&self.pending_remove_locked)
    }
    pub fn get_liquidity_by_principal(&self) -> &HashMap<Principal, LiquidityAmount> {
        &self.liquidity
    }
    pub fn get_locked_add_item(&mut self, index: usize) -> Option<&mut (Principal, TokenAmount)> {
        if index < self.pending_add_locked.len() {
            Some(&mut self.pending_add_locked[index])
        } else {
            None
        }
    }
    pub fn get_locked_remove_item(
        &mut self,
        index: usize,
    ) -> Option<&mut (Principal, TokenAmount)> {
        if index < self.pending_remove_locked.len() {
            Some(&mut self.pending_remove_locked[index])
        } else {
            None
        }
    }
    pub fn get_user_liquidity_mut(
        &mut self,
        user: Principal,
        token: &EnokiToken,
    ) -> &mut StableNat {
        self.liquidity.entry(user).or_default().get_mut(token)
    }
    pub fn remove_zeros(&mut self) {
        self.pending_add_locked
            .retain(|(_, amount)| amount.amount.is_nonzero());
        self.pending_remove_locked
            .retain(|(_, amount)| amount.amount.is_nonzero());
        ic_cdk::println!(
            "[worker] there are now {} and {} pending add and remove liquidity positions",
            self.pending_add_locked.len(),
            self.pending_remove_locked.len()
        );
    }
    pub fn update_user_net_deposits(
        &mut self,
        user: Principal,
        token: &EnokiToken,
        adding: bool,
        mut amount: StableNat,
    ) {
        let deposits = self.user_net_deposits.entry(user).or_default();
        if adding {
            let diff = amount.clone().min(deposits.decreased.get(token).clone());
            if diff.is_nonzero() {
                amount.safe_sub_assign(diff.clone()).unwrap();
                deposits
                    .decreased
                    .get_mut(token)
                    .safe_sub_assign(diff)
                    .unwrap();
            }
            deposits.increased.get_mut(token).add_assign(amount);
        } else {
            let diff = amount.clone().min(deposits.increased.get(token).clone());
            if diff.is_nonzero() {
                amount.safe_sub_assign(diff.clone()).unwrap();
                deposits
                    .increased
                    .get_mut(token)
                    .safe_sub_assign(diff)
                    .unwrap();
            }
            deposits.decreased.get_mut(token).add_assign(amount);
        }
    }
    pub fn apply_traded(&mut self, traded: &HashMap<Principal, LiquidityTrades>) {
        for (user, liquidity) in self.liquidity.iter_mut() {
            if let Some(traded) = traded.get(user) {
                liquidity.add_assign(traded.increased.clone());
                liquidity.safe_sub_assign(traded.decreased.clone()).unwrap();
            }
        }
    }
}
