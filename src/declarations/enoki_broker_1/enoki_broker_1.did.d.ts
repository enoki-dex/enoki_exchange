import type { Principal } from '@dfinity/principal';
export interface AggregateBidAsk {
  'asks' : Array<[bigint, Array<CounterpartyInfo>]>,
  'bids' : Array<[bigint, Array<CounterpartyInfo>]>,
}
export interface AssignedShards { 'token_a' : Principal, 'token_b' : Principal }
export interface CounterpartyInfo {
  'broker' : Principal,
  'user' : Principal,
  'quantity' : Array<number>,
  'price' : bigint,
}
export type EnokiToken = { 'TokenA' : null } |
  { 'TokenB' : null };
export interface InitBrokerParams {
  'liquidity_location' : Principal,
  'other_brokers' : Array<Principal>,
  'supply_token_info' : TokenPairInfo,
  'trading_fees' : TradingFees,
}
export interface LiquidityAmount {
  'token_a' : Array<number>,
  'token_b' : Array<number>,
}
export interface LiquidityTrades {
  'decreased' : LiquidityAmount,
  'increased' : LiquidityAmount,
}
export type MakerTaker = { 'OnlyMaker' : null } |
  { 'OnlyTaker' : null } |
  { 'MakerOrTaker' : null };
export interface OpenOrderStatus {
  'open_orders' : Array<OrderInfo>,
  'pending_cancel' : Array<bigint>,
}
export interface Order { 'info' : OrderInfo, 'state' : OrderState }
export interface OrderInfo {
  'id' : bigint,
  'maker_taker' : MakerTaker,
  'broker' : Principal,
  'limit_price' : bigint,
  'side' : Side,
  'user' : Principal,
  'quantity' : Array<number>,
  'expiration_time' : [] | [bigint],
}
export interface OrderState {
  'status' : OrderStatus,
  'quantity_remaining' : Array<number>,
  'marker_makers' : Array<CounterpartyInfo>,
}
export type OrderStatus = { 'InvalidPrice' : null } |
  { 'InsufficientLiquidity' : null } |
  { 'Cancelled' : null } |
  { 'Completed' : null } |
  { 'Expired' : null } |
  { 'Pending' : null };
export interface PendingTransfer {
  'to' : Principal,
  'token' : EnokiToken,
  'amount' : bigint,
}
export interface RequestForNewLiquidityTarget {
  'extra_liquidity_available' : LiquidityAmount,
  'target' : LiquidityAmount,
}
export interface ResponseAboutLiquidityChanges {
  'added' : LiquidityAmount,
  'traded' : LiquidityTrades,
  'removed' : LiquidityAmount,
}
export interface ShardedTransferNotification {
  'to' : Principal,
  'value' : bigint,
  'data' : string,
  'from' : Principal,
  'fee_charged' : bigint,
  'from_shard' : Principal,
}
export type Side = { 'Buy' : null } |
  { 'Sell' : null };
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
  'fundsSent' : (arg_0: ShardedTransferNotification) => Promise<undefined>,
  'getAccruedFees' : () => Promise<LiquidityAmount>,
  'getAssignedShard' : (arg_0: EnokiToken) => Promise<Principal>,
  'getAssignedShardA' : () => Promise<Principal>,
  'getAssignedShardB' : () => Promise<Principal>,
  'getAssignedShards' : () => Promise<AssignedShards>,
  'getFailedOrders' : () => Promise<Array<Order>>,
  'getManager' : () => Promise<Principal>,
  'getOpenOrders' : (arg_0: Principal) => Promise<OpenOrderStatus>,
  'getOwner' : () => Promise<Principal>,
  'getPastOrders' : (arg_0: Principal) => Promise<Array<Order>>,
  'getTokenInfo' : () => Promise<TokenPairInfo>,
  'getTradingFees' : () => Promise<TradingFees>,
  'initBroker' : (arg_0: InitBrokerParams) => Promise<AssignedShards>,
  'limitOrder' : (arg_0: ShardedTransferNotification) => Promise<undefined>,
  'receiveMarketMakerRewards' : (arg_0: ShardedTransferNotification) => Promise<
      undefined
    >,
  'retrieveOrders' : () => Promise<[Array<OrderInfo>, Array<OrderInfo>]>,
  'sendFunds' : (arg_0: string, arg_1: PendingTransfer) => Promise<undefined>,
  'setFees' : (arg_0: TradingFees) => Promise<undefined>,
  'setManager' : (arg_0: Principal) => Promise<undefined>,
  'setOwner' : (arg_0: Principal) => Promise<undefined>,
  'submitCompletedOrders' : (
      arg_0: Array<Order>,
      arg_1: AggregateBidAsk,
      arg_2: RequestForNewLiquidityTarget,
    ) => Promise<ResponseAboutLiquidityChanges>,
  'swap' : (arg_0: ShardedTransferNotification) => Promise<undefined>,
  'updateUpstreamFees' : () => Promise<undefined>,
}
