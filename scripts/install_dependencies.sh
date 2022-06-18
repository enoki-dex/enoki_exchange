set -a # automatically export all variables
source .env
set +a

dfx identity use default
dfx canister create --all

#### === DEPLOY LOCAL LEDGER =====
#dfx identity new minter
#dfx identity use minter
#MINT_ACC=$(dfx ledger account-id)
#export MINT_ACC
#
#dfx identity use default
#LEDGER_ACC=$(dfx ledger account-id)
#export LEDGER_ACC
#
## Use private api for install
#rm src_dev/ledger/ledger.did
#cp src_dev/ledger/ledger.private.did src_dev/ledger/ledger.did
#
#dfx deploy ledger --argument '(record  {
#    minting_account = "'${MINT_ACC}'";
#    initial_values = vec { record { "'${LEDGER_ACC}'"; record { e8s=100_000_000_000 } }; };
#    send_whitelist = vec {}
#    })'
#LEDGER_ID=$(dfx canister id ledger)
#export LEDGER_ID
#
## Replace with public api
#rm src_dev/ledger/ledger.did
#cp src_dev/ledger/ledger.public.did src_dev/ledger/ledger.did

### === DEPLOY enoki test tokens =====

dfx canister create enoki_wrapped_token
dfx deploy enoki_wrapped_token
dfx canister call enoki_wrapped_token finishInit "(principal \"$(dfx canister id enoki_wrapped_token)\", \"$TOKEN_LOGO_A\", \"$TOKEN_NAME_A\", \"$TOKEN_SYMBOL_A\", $TOKEN_DECIMALS_A:nat8, $TOKEN_FEE_A)"
i=1
num_shards=${NUM_SHARDS:-2}
while [ $i -le $num_shards ]; do
  dfx canister create "enoki_wrapped_token_shard_$i"
  dfx deploy "enoki_wrapped_token_shard_$i"
  dfx canister call "enoki_wrapped_token_shard_$i" finishInit "(principal \"$(dfx canister id enoki_wrapped_token)\", principal \"$(dfx canister id enoki_wrapped_token)\")"
  dfx canister call enoki_wrapped_token "addShard" "(principal \"$(dfx canister id "enoki_wrapped_token_shard_$i")\")"
  true $((i++))
done

dfx canister create enoki_wrapped_token_b
dfx deploy enoki_wrapped_token_b
dfx canister call enoki_wrapped_token_b finishInit "(principal \"$(dfx canister id enoki_wrapped_token_b)\", \"$TOKEN_LOGO_B\", \"$TOKEN_NAME_B\", \"$TOKEN_SYMBOL_B\", $TOKEN_DECIMALS_B:nat8, $TOKEN_FEE_B)"
i=1
num_shards=${NUM_SHARDS:-2}
while [ $i -le $num_shards ]; do
  dfx canister create "enoki_wrapped_token_shard_b_$i"
  dfx deploy "enoki_wrapped_token_shard_b_$i"
  dfx canister call "enoki_wrapped_token_shard_b_$i" finishInit "(principal \"$(dfx canister id enoki_wrapped_token_b)\", principal \"$(dfx canister id enoki_wrapped_token_b)\")"
  dfx canister call enoki_wrapped_token_b "addShard" "(principal \"$(dfx canister id "enoki_wrapped_token_shard_b_$i")\")"
  true $((i++))
done

### === DEPLOY INTERNET IDENTITY =====

II_ENV=development dfx deploy internet_identity --argument '(null)'
