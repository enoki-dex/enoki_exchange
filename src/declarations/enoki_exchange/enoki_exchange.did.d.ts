import type { Principal } from '@dfinity/principal';
export interface AssignedShards { 'token_a' : Principal, 'token_b' : Principal }
export interface BidAskCurve {
  'asks' : Array<[bigint, bigint]>,
  'bids' : Array<[bigint, bigint]>,
  'num_decimals' : bigint,
}
export interface LastPricePoint {
  'time' : bigint,
  'price_was_lifted' : boolean,
  'price' : number,
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
  'finishInit' : (arg_0: Principal, arg_1: Principal, arg_2: bigint) => Promise<
      undefined
    >,
  'getAssignedBroker' : (arg_0: Principal) => Promise<Principal>,
  'getAssignedShardA' : () => Promise<Principal>,
  'getAssignedShardB' : () => Promise<Principal>,
  'getAssignedShards' : () => Promise<AssignedShards>,
  'getBidAskCurve' : () => Promise<BidAskCurve>,
  'getBrokerIds' : () => Promise<Array<Principal>>,
  'getLiquidityLocation' : () => Promise<Principal>,
  'getOwner' : () => Promise<Principal>,
  'getPriceHistory' : () => Promise<Array<LastPricePoint>>,
  'getTokenInfo' : () => Promise<TokenPairInfo>,
  'getTradingFees' : () => Promise<TradingFees>,
  'initPool' : (arg_0: Principal) => Promise<undefined>,
  'register' : (arg_0: Principal) => Promise<Principal>,
  'setFees' : (
      arg_0: bigint,
      arg_1: bigint,
      arg_2: number,
      arg_3: number,
      arg_4: number,
    ) => Promise<undefined>,
  'setOwner' : (arg_0: Principal) => Promise<undefined>,
  'triggerRun' : () => Promise<[] | [bigint]>,
  'whoami' : () => Promise<Principal>,
  'whoisanon' : () => Promise<Principal>,
}
