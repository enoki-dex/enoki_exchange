export const idlFactory = ({ IDL }) => {
  const ShardedTransferNotification = IDL.Record({
    'to' : IDL.Principal,
    'value' : IDL.Nat,
    'data' : IDL.Text,
    'from' : IDL.Principal,
    'fee_charged' : IDL.Nat,
    'from_shard' : IDL.Principal,
  });
  const AssignedShards = IDL.Record({
    'token_a' : IDL.Principal,
    'token_b' : IDL.Principal,
  });
  const LiquidityAmountNat = IDL.Record({
    'token_a' : IDL.Nat,
    'token_b' : IDL.Nat,
  });
  const LiquidityTradesNat = IDL.Record({
    'decreased' : LiquidityAmountNat,
    'increased' : LiquidityAmountNat,
  });
  const TokenInfo = IDL.Record({ 'principal' : IDL.Principal });
  const TokenPairInfo = IDL.Record({
    'token_a' : TokenInfo,
    'token_b' : TokenInfo,
    'price_number_of_decimals' : IDL.Nat64,
  });
  return IDL.Service({
    'addBroker' : IDL.Func([IDL.Principal], [], []),
    'addLiquidity' : IDL.Func([ShardedTransferNotification], [IDL.Text], []),
    'finishInit' : IDL.Func([IDL.Principal], [], []),
    'getAssignedShardA' : IDL.Func([], [IDL.Principal], ['query']),
    'getAssignedShardB' : IDL.Func([], [IDL.Principal], ['query']),
    'getAssignedShards' : IDL.Func([], [AssignedShards], ['query']),
    'getLiquidity' : IDL.Func([IDL.Principal], [LiquidityAmountNat], ['query']),
    'getManager' : IDL.Func([], [IDL.Principal], ['query']),
    'getNetDeposits' : IDL.Func(
        [IDL.Principal],
        [LiquidityTradesNat],
        ['query'],
      ),
    'getOwner' : IDL.Func([], [IDL.Principal], ['query']),
    'getShardsToAddLiquidity' : IDL.Func([], [AssignedShards], ['query']),
    'getTokenInfo' : IDL.Func([], [TokenPairInfo], ['query']),
    'initWorker' : IDL.Func([TokenPairInfo], [AssignedShards], []),
    'isUserRegistered' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'register' : IDL.Func([IDL.Principal], [], []),
    'removeAllLiquidity' : IDL.Func([], [], []),
    'removeLiquidity' : IDL.Func([LiquidityAmountNat], [], []),
    'setManager' : IDL.Func([IDL.Principal], [], []),
    'setOwner' : IDL.Func([IDL.Principal], [], []),
    'triggerHeartbeat' : IDL.Func([], [IDL.Opt(IDL.Nat64)], []),
  });
};
export const init = ({ IDL }) => { return []; };
