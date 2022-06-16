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
  return IDL.Service({
    'addBroker' : IDL.Func([IDL.Principal], [], []),
    'finishInit' : IDL.Func([IDL.Principal, IDL.Principal, IDL.Nat64], [], []),
    'getAssignedShardA' : IDL.Func([], [IDL.Principal], ['query']),
    'getAssignedShardB' : IDL.Func([], [IDL.Principal], ['query']),
    'getAssignedShards' : IDL.Func([], [AssignedShards], ['query']),
    'getBrokerIds' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'getLiquidityLocation' : IDL.Func([], [IDL.Principal], []),
    'getOwner' : IDL.Func([], [IDL.Principal], ['query']),
    'getRandomBroker' : IDL.Func([], [IDL.Principal], []),
    'getTokenInfo' : IDL.Func([], [TokenPairInfo], ['query']),
    'getTradingFees' : IDL.Func([], [TradingFees], ['query']),
    'initPool' : IDL.Func([IDL.Principal], [], []),
    'setFees' : IDL.Func(
        [IDL.Nat, IDL.Nat, IDL.Float64, IDL.Float64, IDL.Float64],
        [],
        [],
      ),
    'setOwner' : IDL.Func([IDL.Principal], [], []),
    'triggerRun' : IDL.Func([], [], []),
    'whoami' : IDL.Func([], [IDL.Principal], ['query']),
    'whoisanon' : IDL.Func([], [IDL.Principal], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
