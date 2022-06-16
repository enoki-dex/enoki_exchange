export const idlFactory = ({ IDL }) => {
  const TxError = IDL.Variant({
    'UnderlyingTransferFailure' : IDL.Null,
    'TransferCallbackError' : IDL.Text,
    'InsufficientBalance' : IDL.Null,
    'TransferValueTooSmall' : IDL.Null,
    'Unauthorized' : IDL.Null,
    'AccountDoesNotExist' : IDL.Record({
      'user' : IDL.Text,
      'shard' : IDL.Text,
    }),
    'ShardDoesNotExist' : IDL.Null,
    'AccountAlreadyExists' : IDL.Null,
    'Other' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : TxError });
  const Metadata = IDL.Record({
    'underlying_token' : IDL.Principal,
    'decimals' : IDL.Nat8,
    'logo' : IDL.Text,
    'name' : IDL.Text,
    'symbol' : IDL.Text,
  });
  const Shard = IDL.Record({
    'id' : IDL.Principal,
    'num_accounts' : IDL.Nat64,
  });
  const Stats = IDL.Record({
    'fee' : IDL.Nat,
    'deploy_time' : IDL.Nat64,
    'owner' : IDL.Principal,
    'cycles' : IDL.Nat64,
    'total_supply' : IDL.Nat,
  });
  return IDL.Service({
    'addShard' : IDL.Func([IDL.Principal], [Result], []),
    'balanceOf' : IDL.Func([IDL.Principal], [IDL.Nat], ['query']),
    'decimals' : IDL.Func([], [IDL.Nat8], ['query']),
    'getAccruedFees' : IDL.Func([], [IDL.Nat], ['query']),
    'getAssignedShardId' : IDL.Func(
        [IDL.Principal],
        [IDL.Principal],
        ['query'],
      ),
    'getFee' : IDL.Func([], [IDL.Nat], ['query']),
    'getLogo' : IDL.Func([], [IDL.Text], ['query']),
    'getMetadata' : IDL.Func([], [Metadata], ['query']),
    'getShardIds' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'getShardIdsUpdate' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'getShardsInfo' : IDL.Func([], [IDL.Vec(Shard)], ['query']),
    'name' : IDL.Func([], [IDL.Text], ['query']),
    'owner' : IDL.Func([], [IDL.Principal], ['query']),
    'register' : IDL.Func([IDL.Principal], [IDL.Principal], []),
    'setFee' : IDL.Func([IDL.Nat], [Result], []),
    'setLogo' : IDL.Func([IDL.Text], [Result], []),
    'setOwner' : IDL.Func([IDL.Principal], [Result], []),
    'stats' : IDL.Func([], [Stats], ['query']),
    'symbol' : IDL.Func([], [IDL.Text], ['query']),
    'totalSupply' : IDL.Func([], [IDL.Nat], ['query']),
    'transfer' : IDL.Func([IDL.Principal, IDL.Nat], [], []),
  });
};
export const init = ({ IDL }) => {
  return [IDL.Principal, IDL.Text, IDL.Text, IDL.Text, IDL.Nat8, IDL.Nat];
};
