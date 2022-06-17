import type { Principal } from '@dfinity/principal';
export interface AssignedShards { 'token_a' : Principal, 'token_b' : Principal }
export interface LiquidityAmountNat { 'token_a' : bigint, 'token_b' : bigint }
export interface LiquidityTradesNat {
  'decreased' : LiquidityAmountNat,
  'increased' : LiquidityAmountNat,
}
export interface ShardedTransferNotification {
  'to' : Principal,
  'value' : bigint,
  'data' : string,
  'from' : Principal,
  'fee_charged' : bigint,
  'from_shard' : Principal,
}
export interface TokenInfo { 'principal' : Principal }
export interface TokenPairInfo {
  'token_a' : TokenInfo,
  'token_b' : TokenInfo,
  'price_number_of_decimals' : bigint,
}
export interface _SERVICE {
  'addBroker' : (arg_0: Principal) => Promise<undefined>,
  'addLiquidity' : (arg_0: ShardedTransferNotification) => Promise<undefined>,
  'finishInit' : (arg_0: Principal) => Promise<undefined>,
  'getAssignedShardA' : () => Promise<Principal>,
  'getAssignedShardB' : () => Promise<Principal>,
  'getAssignedShards' : () => Promise<AssignedShards>,
  'getLiquidity' : (arg_0: Principal) => Promise<LiquidityAmountNat>,
  'getManager' : () => Promise<Principal>,
  'getNetDeposits' : (arg_0: Principal) => Promise<LiquidityTradesNat>,
  'getOwner' : () => Promise<Principal>,
  'getShardsToAddLiquidity' : () => Promise<AssignedShards>,
  'getTokenInfo' : () => Promise<TokenPairInfo>,
  'initWorker' : (arg_0: TokenPairInfo) => Promise<AssignedShards>,
  'isUserRegistered' : (arg_0: Principal) => Promise<boolean>,
  'register' : (arg_0: Principal) => Promise<undefined>,
  'removeAllLiquidity' : () => Promise<undefined>,
  'removeLiquidity' : (arg_0: LiquidityAmountNat) => Promise<undefined>,
  'setManager' : (arg_0: Principal) => Promise<undefined>,
  'setOwner' : (arg_0: Principal) => Promise<undefined>,
  'triggerHeartbeat' : () => Promise<[] | [bigint]>,
}
