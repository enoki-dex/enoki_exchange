import type { Principal } from '@dfinity/principal';
export interface AssignedShards { 'token_a' : Principal, 'token_b' : Principal }
export interface LiquidityAmount {
  'token_a' : Array<number>,
  'token_b' : Array<number>,
}
export interface LiquidityAmountNat { 'token_a' : bigint, 'token_b' : bigint }
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
  'getOwner' : () => Promise<Principal>,
  'getShardsToAddLiquidity' : () => Promise<AssignedShards>,
  'getTokenInfo' : () => Promise<TokenPairInfo>,
  'initWorker' : (arg_0: TokenPairInfo) => Promise<AssignedShards>,
  'removeAllLiquidity' : () => Promise<undefined>,
  'removeLiquidity' : (arg_0: LiquidityAmount) => Promise<undefined>,
  'setManager' : (arg_0: Principal) => Promise<undefined>,
  'setOwner' : (arg_0: Principal) => Promise<undefined>,
  'triggerHeartbeat' : () => Promise<[] | [bigint]>,
}