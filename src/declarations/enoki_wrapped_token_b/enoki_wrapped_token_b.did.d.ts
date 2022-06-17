import type { Principal } from '@dfinity/principal';
export interface Metadata {
  'underlying_token' : Principal,
  'decimals' : number,
  'logo' : string,
  'name' : string,
  'symbol' : string,
}
export interface Shard { 'id' : Principal, 'num_accounts' : bigint }
export interface Stats {
  'fee' : bigint,
  'deploy_time' : bigint,
  'owner' : Principal,
  'cycles' : bigint,
  'total_supply' : bigint,
}
export interface _SERVICE {
  'addShard' : (arg_0: Principal) => Promise<undefined>,
  'balanceOf' : (arg_0: Principal) => Promise<bigint>,
  'decimals' : () => Promise<number>,
  'finishInit' : (
      arg_0: Principal,
      arg_1: string,
      arg_2: string,
      arg_3: string,
      arg_4: number,
      arg_5: bigint,
    ) => Promise<undefined>,
  'fixSiblings' : () => Promise<undefined>,
  'getAccruedFees' : () => Promise<bigint>,
  'getAssignedShardId' : (arg_0: Principal) => Promise<Principal>,
  'getFee' : () => Promise<bigint>,
  'getLogo' : () => Promise<string>,
  'getMetadata' : () => Promise<Metadata>,
  'getShardIds' : () => Promise<Array<Principal>>,
  'getShardIdsUpdate' : () => Promise<Array<Principal>>,
  'getShardsInfo' : () => Promise<Array<Shard>>,
  'name' : () => Promise<string>,
  'owner' : () => Promise<Principal>,
  'register' : (arg_0: Principal) => Promise<Principal>,
  'setFee' : (arg_0: bigint) => Promise<undefined>,
  'setLogo' : (arg_0: string) => Promise<undefined>,
  'setOwner' : (arg_0: Principal) => Promise<undefined>,
  'stats' : () => Promise<Stats>,
  'symbol' : () => Promise<string>,
  'totalSupply' : () => Promise<bigint>,
  'transfer' : (arg_0: Principal, arg_1: bigint) => Promise<undefined>,
}
