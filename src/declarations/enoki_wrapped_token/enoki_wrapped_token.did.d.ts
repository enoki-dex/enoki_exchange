import type { Principal } from '@dfinity/principal';
export interface Metadata {
  'underlying_token' : Principal,
  'decimals' : number,
  'logo' : string,
  'name' : string,
  'symbol' : string,
}
export type Result = { 'Ok' : null } |
  { 'Err' : TxError };
export interface Shard { 'id' : Principal, 'num_accounts' : bigint }
export interface Stats {
  'fee' : bigint,
  'deploy_time' : bigint,
  'owner' : Principal,
  'cycles' : bigint,
  'total_supply' : bigint,
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
  'addShard' : (arg_0: Principal) => Promise<Result>,
  'balanceOf' : (arg_0: Principal) => Promise<bigint>,
  'decimals' : () => Promise<number>,
  'getAccruedFees' : () => Promise<bigint>,
  'getAssignedShardId' : (arg_0: Principal) => Promise<Principal>,
  'getFee' : () => Promise<bigint>,
  'getLogo' : () => Promise<string>,
  'getMetadata' : () => Promise<Metadata>,
  'getShardIds' : () => Promise<Array<Principal>>,
  'getShardsInfo' : () => Promise<Array<Shard>>,
  'name' : () => Promise<string>,
  'owner' : () => Promise<Principal>,
  'register' : (arg_0: Principal) => Promise<Principal>,
  'setFee' : (arg_0: bigint) => Promise<Result>,
  'setLogo' : (arg_0: string) => Promise<Result>,
  'setOwner' : (arg_0: Principal) => Promise<Result>,
  'stats' : () => Promise<Stats>,
  'symbol' : () => Promise<string>,
  'totalSupply' : () => Promise<bigint>,
  'transfer' : (arg_0: Principal, arg_1: bigint) => Promise<undefined>,
}
