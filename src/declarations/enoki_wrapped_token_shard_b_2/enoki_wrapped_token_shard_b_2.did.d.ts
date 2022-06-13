import type { Principal } from '@dfinity/principal';
export interface ManagerContractData {
  'fee' : bigint,
  'deploy_time' : bigint,
  'underlying_token' : Principal,
  'owner' : Principal,
  'sibling_shards' : Array<Principal>,
  'manager_contract' : Principal,
}
export type Result = { 'Ok' : null } |
  { 'Err' : TxError };
export interface ShardedTransferNotification {
  'to' : Principal,
  'value' : bigint,
  'data' : string,
  'from' : Principal,
  'fee_charged' : bigint,
  'from_shard' : Principal,
}
export type TxError = { 'UnderlyingTransferFailure' : null } |
  { 'TransferCallbackError' : string } |
  { 'InsufficientBalance' : null } |
  { 'TransferValueTooSmall' : null } |
  { 'Unauthorized' : null } |
  { 'AccountDoesNotExist' : null } |
  { 'ShardDoesNotExist' : null } |
  { 'AccountAlreadyExists' : null } |
  { 'Other' : string };
export interface _SERVICE {
  'addSiblingShard' : (arg_0: Principal) => Promise<Result>,
  'addSpender' : (arg_0: Principal) => Promise<undefined>,
  'createAccount' : (arg_0: Principal) => Promise<Result>,
  'getAccruedFees' : () => Promise<bigint>,
  'getFee' : () => Promise<bigint>,
  'getManagementDetails' : () => Promise<ManagerContractData>,
  'getOwner' : () => Promise<Principal>,
  'initShard' : (
      arg_0: Principal,
      arg_1: Array<Principal>,
      arg_2: bigint,
    ) => Promise<Result>,
  'mint' : (arg_0: bigint) => Promise<undefined>,
  'removeSiblingShard' : (arg_0: Principal) => Promise<Result>,
  'removeSpender' : (arg_0: Principal) => Promise<undefined>,
  'setFee' : (arg_0: bigint) => Promise<Result>,
  'setOwner' : (arg_0: Principal) => Promise<Result>,
  'shardBalanceOf' : (arg_0: Principal) => Promise<bigint>,
  'shardGetSupply' : () => Promise<bigint>,
  'shardReceiveTransfer' : (arg_0: Principal, arg_1: bigint) => Promise<Result>,
  'shardReceiveTransferAndCall' : (
      arg_0: ShardedTransferNotification,
      arg_1: Principal,
      arg_2: string,
    ) => Promise<Result>,
  'shardSpend' : (
      arg_0: Principal,
      arg_1: Principal,
      arg_2: Principal,
      arg_3: bigint,
    ) => Promise<undefined>,
  'shardSpendAndCall' : (
      arg_0: Principal,
      arg_1: Principal,
      arg_2: Principal,
      arg_3: bigint,
      arg_4: Principal,
      arg_5: string,
      arg_6: string,
    ) => Promise<undefined>,
  'shardTransfer' : (
      arg_0: Principal,
      arg_1: Principal,
      arg_2: bigint,
    ) => Promise<undefined>,
  'shardTransferAndCall' : (
      arg_0: Principal,
      arg_1: Principal,
      arg_2: bigint,
      arg_3: Principal,
      arg_4: string,
      arg_5: string,
    ) => Promise<undefined>,
  'transferFromManager' : (
      arg_0: Principal,
      arg_1: Principal,
      arg_2: Principal,
      arg_3: bigint,
    ) => Promise<Result>,
  'unwrap' : (arg_0: bigint, arg_1: Principal) => Promise<undefined>,
  'wrap' : (arg_0: bigint) => Promise<undefined>,
}
