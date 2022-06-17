export const idlFactory = ({ IDL }) => {
  const ManagerContractData = IDL.Record({
    'fee' : IDL.Nat,
    'deploy_time' : IDL.Nat64,
    'underlying_token' : IDL.Principal,
    'owner' : IDL.Principal,
    'sibling_shards' : IDL.Vec(IDL.Principal),
    'manager_contract' : IDL.Principal,
  });
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
  const ShardedTransferNotification = IDL.Record({
    'to' : IDL.Principal,
    'value' : IDL.Nat,
    'data' : IDL.Text,
    'from' : IDL.Principal,
    'fee_charged' : IDL.Nat,
    'from_shard' : IDL.Principal,
  });
  return IDL.Service({
    'addSiblingShard' : IDL.Func([IDL.Principal], [], []),
    'addSpender' : IDL.Func([IDL.Principal], [], []),
    'createAccount' : IDL.Func([IDL.Principal], [], []),
    'finishInit' : IDL.Func([IDL.Principal, IDL.Principal], [], []),
    'getAccruedFees' : IDL.Func([], [IDL.Nat], ['query']),
    'getFee' : IDL.Func([], [IDL.Nat], ['query']),
    'getManagementDetails' : IDL.Func([], [ManagerContractData], ['query']),
    'getOwner' : IDL.Func([], [IDL.Principal], ['query']),
    'initShard' : IDL.Func(
        [IDL.Principal, IDL.Vec(IDL.Principal), IDL.Nat],
        [],
        [],
      ),
    'mint' : IDL.Func([IDL.Nat], [], []),
    'removeSiblingShard' : IDL.Func([IDL.Principal], [], []),
    'removeSpender' : IDL.Func([IDL.Principal], [], []),
    'setFee' : IDL.Func([IDL.Nat], [Result], []),
    'setOwner' : IDL.Func([IDL.Principal], [Result], []),
    'shardBalanceOf' : IDL.Func([IDL.Principal], [IDL.Nat], ['query']),
    'shardGetSupply' : IDL.Func([], [IDL.Nat], ['query']),
    'shardReceiveTransfer' : IDL.Func([IDL.Principal, IDL.Nat], [], []),
    'shardReceiveTransferAndCall' : IDL.Func(
        [ShardedTransferNotification, IDL.Principal, IDL.Text],
        [IDL.Text],
        [],
      ),
    'shardSpend' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Principal, IDL.Nat],
        [],
        [],
      ),
    'shardSpendAndCall' : IDL.Func(
        [
          IDL.Principal,
          IDL.Principal,
          IDL.Principal,
          IDL.Nat,
          IDL.Principal,
          IDL.Text,
          IDL.Text,
        ],
        [IDL.Text],
        [],
      ),
    'shardTransfer' : IDL.Func([IDL.Principal, IDL.Principal, IDL.Nat], [], []),
    'shardTransferAndCall' : IDL.Func(
        [
          IDL.Principal,
          IDL.Principal,
          IDL.Nat,
          IDL.Principal,
          IDL.Text,
          IDL.Text,
        ],
        [IDL.Text],
        [],
      ),
    'transferFromManager' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Principal, IDL.Nat],
        [],
        [],
      ),
    'unwrap' : IDL.Func([IDL.Nat, IDL.Principal], [], []),
    'wrap' : IDL.Func([IDL.Nat], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
