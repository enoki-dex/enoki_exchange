export const idlFactory = ({ IDL }) => {
  const AssignedShards = IDL.Record({
    'token_a' : IDL.Principal,
    'token_b' : IDL.Principal,
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
  const LiquidityAmount = IDL.Record({
    'token_a' : IDL.Vec(IDL.Nat8),
    'token_b' : IDL.Vec(IDL.Nat8),
  });
  const LiquidityTrades = IDL.Record({
    'decreased' : LiquidityAmount,
    'increased' : LiquidityAmount,
  });
  return IDL.Service({
    'addBroker' : IDL.Func([IDL.Principal], [], []),
    'finishInit' : IDL.Func([IDL.Principal], [], []),
    'getAssignedShardA' : IDL.Func([], [IDL.Principal], ['query']),
    'getAssignedShardB' : IDL.Func([], [IDL.Principal], ['query']),
    'getAssignedShards' : IDL.Func([], [AssignedShards], ['query']),
    'getManager' : IDL.Func([], [IDL.Principal], ['query']),
    'getOwner' : IDL.Func([], [IDL.Principal], ['query']),
    'getTokenInfo' : IDL.Func([], [TokenPairInfo], ['query']),
    'getTradingFees' : IDL.Func([], [TradingFees], ['query']),
    'getUpdatedLiquidity' : IDL.Func(
        [],
        [LiquidityAmount, LiquidityAmount],
        [],
      ),
    'getWorker' : IDL.Func([], [IDL.Principal], ['query']),
    'initLiquidityPool' : IDL.Func([TokenPairInfo], [IDL.Principal], []),
    'initWorker' : IDL.Func([IDL.Principal], [], []),
    'resolveLiquidity' : IDL.Func(
        [LiquidityAmount, LiquidityAmount, LiquidityTrades],
        [],
        [],
      ),
    'setManager' : IDL.Func([IDL.Principal], [], []),
    'setOwner' : IDL.Func([IDL.Principal], [], []),
    'updateLiquidity' : IDL.Func(
        [LiquidityAmount, LiquidityAmount],
        [LiquidityAmount, LiquidityAmount, LiquidityTrades],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
