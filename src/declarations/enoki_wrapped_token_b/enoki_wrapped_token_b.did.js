export const idlFactory = ({ IDL }) => {
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
    'addShard' : IDL.Func([IDL.Principal], [], []),
    'balanceOf' : IDL.Func([IDL.Principal], [IDL.Nat], []),
    'decimals' : IDL.Func([], [IDL.Nat8], ['query']),
    'finishInit' : IDL.Func(
        [IDL.Principal, IDL.Text, IDL.Text, IDL.Text, IDL.Nat8, IDL.Nat],
        [],
        [],
      ),
    'fixSiblings' : IDL.Func([], [], []),
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
    'setFee' : IDL.Func([IDL.Nat], [], []),
    'setLogo' : IDL.Func([IDL.Text], [], []),
    'setOwner' : IDL.Func([IDL.Principal], [], []),
    'stats' : IDL.Func([], [Stats], []),
    'symbol' : IDL.Func([], [IDL.Text], ['query']),
    'totalSupply' : IDL.Func([], [IDL.Nat], []),
    'transfer' : IDL.Func([IDL.Principal, IDL.Nat], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
