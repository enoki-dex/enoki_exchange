export const idlFactory = ({ IDL }) => {
  const ShardedTransferNotification = IDL.Record({
    'to' : IDL.Principal,
    'value' : IDL.Nat,
    'data' : IDL.Text,
    'from' : IDL.Principal,
    'fee_charged' : IDL.Nat,
    'from_shard' : IDL.Principal,
  });
  const LiquidityAmountNat = IDL.Record({
    'token_a' : IDL.Nat,
    'token_b' : IDL.Nat,
  });
  const LiquidityAmount = IDL.Record({
    'token_a' : IDL.Vec(IDL.Nat8),
    'token_b' : IDL.Vec(IDL.Nat8),
  });
  const AssignedShards = IDL.Record({
    'token_a' : IDL.Principal,
    'token_b' : IDL.Principal,
  });
  const Side = IDL.Variant({ 'Buy' : IDL.Null, 'Sell' : IDL.Null });
  const MakerTaker = IDL.Variant({
    'OnlyMaker' : IDL.Null,
    'OnlyTaker' : IDL.Null,
    'MakerOrTaker' : IDL.Null,
  });
  const OrderInfo = IDL.Record({
    'id' : IDL.Nat64,
    'maker_taker' : MakerTaker,
    'broker' : IDL.Principal,
    'limit_price' : IDL.Nat64,
    'side' : Side,
    'user' : IDL.Principal,
    'quantity' : IDL.Vec(IDL.Nat8),
    'expiration_time' : IDL.Opt(IDL.Nat64),
  });
  const OrderStatus = IDL.Variant({
    'InvalidPrice' : IDL.Null,
    'InsufficientLiquidity' : IDL.Null,
    'Cancelled' : IDL.Null,
    'Completed' : IDL.Null,
    'Expired' : IDL.Null,
    'Pending' : IDL.Null,
  });
  const CounterpartyInfo = IDL.Record({
    'broker' : IDL.Principal,
    'user' : IDL.Principal,
    'quantity' : IDL.Vec(IDL.Nat8),
    'price' : IDL.Nat64,
  });
  const OrderState = IDL.Record({
    'status' : OrderStatus,
    'quantity_remaining' : IDL.Vec(IDL.Nat8),
    'marker_makers' : IDL.Vec(CounterpartyInfo),
  });
  const Order = IDL.Record({ 'info' : OrderInfo, 'state' : OrderState });
  const OpenOrderStatus = IDL.Record({
    'open_orders' : IDL.Vec(OrderInfo),
    'pending_cancel' : IDL.Vec(IDL.Nat64),
  });
  const TokenInfo = IDL.Record({ 'principal' : IDL.Principal });
  const TokenPairInfo = IDL.Record({
    'token_a' : TokenInfo,
    'token_b' : TokenInfo,
    'price_number_of_decimals' : IDL.Nat64,
  });
  const TradingFees = IDL.Record({
    'limit_order_taker_fee' : IDL.Float64,
    'swap_market_maker_reward' : IDL.Float64,
    'swap_fee' : IDL.Float64,
    'token_a_deposit_fee' : IDL.Vec(IDL.Nat8),
    'token_b_deposit_fee' : IDL.Vec(IDL.Nat8),
  });
  const InitBrokerParams = IDL.Record({
    'liquidity_location' : IDL.Principal,
    'other_brokers' : IDL.Vec(IDL.Principal),
    'supply_token_info' : TokenPairInfo,
    'trading_fees' : TradingFees,
  });
  const EnokiToken = IDL.Variant({ 'TokenA' : IDL.Null, 'TokenB' : IDL.Null });
  const FirstTransfer = IDL.Record({
    'to' : IDL.Principal,
    'token' : EnokiToken,
    'to_shard' : IDL.Principal,
    'amount' : IDL.Nat,
    'user_for_shard_id_to_retrieve' : IDL.Principal,
  });
  const AggregateBidAsk = IDL.Record({
    'asks' : IDL.Vec(IDL.Tuple(IDL.Nat64, IDL.Vec(CounterpartyInfo))),
    'bids' : IDL.Vec(IDL.Tuple(IDL.Nat64, IDL.Vec(CounterpartyInfo))),
  });
  const RequestForNewLiquidityTarget = IDL.Record({
    'extra_liquidity_available' : LiquidityAmount,
    'target' : LiquidityAmount,
  });
  const LiquidityTrades = IDL.Record({
    'decreased' : LiquidityAmount,
    'increased' : LiquidityAmount,
  });
  const ResponseAboutLiquidityChanges = IDL.Record({
    'added' : LiquidityAmount,
    'traded' : LiquidityTrades,
    'removed' : LiquidityAmount,
  });
  return IDL.Service({
    'addBroker' : IDL.Func([IDL.Principal], [], []),
    'addUser' : IDL.Func([IDL.Principal], [], []),
    'finishInit' : IDL.Func([IDL.Principal], [], []),
    'fundsSent' : IDL.Func([ShardedTransferNotification], [], []),
    'getAccruedExtraRewards' : IDL.Func(
        [IDL.Principal],
        [LiquidityAmountNat],
        ['query'],
      ),
    'getAccruedFees' : IDL.Func([], [LiquidityAmount], ['query']),
    'getAssignedShardA' : IDL.Func([], [IDL.Principal], ['query']),
    'getAssignedShardB' : IDL.Func([], [IDL.Principal], ['query']),
    'getAssignedShards' : IDL.Func([], [AssignedShards], ['query']),
    'getExpectedSwapPrice' : IDL.Func(
        [Side, IDL.Nat],
        [IDL.Float64],
        ['query'],
      ),
    'getFailedOrders' : IDL.Func([], [IDL.Vec(Order)], ['query']),
    'getManager' : IDL.Func([], [IDL.Principal], ['query']),
    'getOpenOrders' : IDL.Func([IDL.Principal], [OpenOrderStatus], ['query']),
    'getOwner' : IDL.Func([], [IDL.Principal], ['query']),
    'getPastOrders' : IDL.Func([IDL.Principal], [IDL.Vec(Order)], ['query']),
    'getTokenInfo' : IDL.Func([], [TokenPairInfo], ['query']),
    'getTradingFees' : IDL.Func([], [TradingFees], ['query']),
    'initBroker' : IDL.Func([InitBrokerParams], [AssignedShards], []),
    'isUserRegistered' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'limitOrder' : IDL.Func([ShardedTransferNotification], [], []),
    'receiveMarketMakerRewards' : IDL.Func(
        [ShardedTransferNotification],
        [],
        [],
      ),
    'register' : IDL.Func([IDL.Principal], [], []),
    'retrieveOrders' : IDL.Func(
        [],
        [IDL.Vec(OrderInfo), IDL.Vec(OrderInfo)],
        [],
      ),
    'sendFunds' : IDL.Func([IDL.Text, FirstTransfer], [], []),
    'setFees' : IDL.Func([TradingFees], [], []),
    'setManager' : IDL.Func([IDL.Principal], [], []),
    'setOwner' : IDL.Func([IDL.Principal], [], []),
    'submitCompletedOrders' : IDL.Func(
        [IDL.Vec(Order), AggregateBidAsk, RequestForNewLiquidityTarget],
        [ResponseAboutLiquidityChanges],
        [],
      ),
    'swap' : IDL.Func([ShardedTransferNotification], [], []),
    'updateUpstreamFees' : IDL.Func([], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
