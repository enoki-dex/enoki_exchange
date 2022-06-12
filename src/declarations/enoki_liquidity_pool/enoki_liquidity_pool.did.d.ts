import type { Principal } from '@dfinity/principal';
export interface AssignedShards { 'token_a' : Principal, 'token_b' : Principal }
export interface LiquidityAmount {
  'token_a' : Array<number>,
  'token_b' : Array<number>,
}
export interface LiquidityTrades {
  'decreased' : LiquidityAmount,
  'increased' : LiquidityAmount,
}
export interface TokenInfo { 'principal' : Principal }
export interface TokenPairInfo {
  'token_a' : TokenInfo,
  'token_b' : TokenInfo,
  'price_number_of_decimals' : bigint,
}
export interface TradingFees {
  'limit_order_taker_fee' : number,
  'swap_market_maker_reward' : number,
  'swap_fee' : number,
  'token_a_deposit_fee' : Array<number>,
  'token_b_deposit_fee' : Array<number>,
}
export interface _SERVICE {
  'addBroker' : (arg_0: Principal) => Promise<undefined>,
  'finishInit' : (arg_0: Principal) => Promise<undefined>,
  'getAssignedShardA' : () => Promise<Principal>,
  'getAssignedShardB' : () => Promise<Principal>,
  'getAssignedShards' : () => Promise<AssignedShards>,
  'getManager' : () => Promise<Principal>,
  'getOwner' : () => Promise<Principal>,
  'getTokenInfo' : () => Promise<TokenPairInfo>,
  'getTradingFees' : () => Promise<TradingFees>,
  'getUpdatedLiquidity' : () => Promise<[LiquidityAmount, LiquidityAmount]>,
  'getWorker' : () => Promise<Principal>,
  'initLiquidityPool' : (arg_0: TokenPairInfo) => Promise<Principal>,
  'initWorker' : (arg_0: Principal) => Promise<undefined>,
  'resolveLiquidity' : (
      arg_0: LiquidityAmount,
      arg_1: LiquidityAmount,
      arg_2: LiquidityTrades,
    ) => Promise<undefined>,
  'setManager' : (arg_0: Principal) => Promise<undefined>,
  'setOwner' : (arg_0: Principal) => Promise<undefined>,
  'updateLiquidity' : (
      arg_0: LiquidityAmount,
      arg_1: LiquidityAmount,
    ) => Promise<[LiquidityAmount, LiquidityAmount, LiquidityTrades]>,
}
