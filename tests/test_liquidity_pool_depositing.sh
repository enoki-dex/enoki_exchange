. "$(dirname "$0")"/setup.sh

if [ -z "$APP_TOKEN_A" ]; then
  APP_TOKEN_A=$(dfx canister id enoki_wrapped_token)
  export APP_TOKEN_A
  APP_TOKEN_B=$(dfx canister id enoki_wrapped_token_b)
  export APP_TOKEN_B
    echo "setting tokens A=$APP_TOKEN_A B=$APP_TOKEN_B";
  else
    echo "tokens already set A=$APP_TOKEN_A B=$APP_TOKEN_B";
fi


start "creating users"
dfx identity new pooler1 || true
dfx identity new pooler2 || true
pooler1=$(dfx --identity pooler1 identity get-principal)
info "pooler1: $pooler1"
pooler2=$(dfx --identity pooler2 identity get-principal)
info "pooler2: $pooler2"
end

start "fund users"
dfx identity use default
mint_amount="1_000_000_000"
assigned_shard_1=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$pooler1\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_1=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$pooler1\")" | grep -oE "$REGEX_PRINCIPAL")
info "pooler1 assigned shards: $assigned_shard_1 / $assigned_shard_b_1"
assigned_shard_2=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$pooler2\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_2=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$pooler2\")" | grep -oE "$REGEX_PRINCIPAL")
info "pooler2 assigned shards: $assigned_shard_2 / $assigned_shard_b_2"
if [ "0 : nat" == "$(dfx canister call "$assigned_shard_2" shardBalanceOf "(principal \"$pooler2\")" | grep -oE "$REGEX_NAT")" ]; then
  dfx --identity pooler1 canister call "$assigned_shard_1" mint "($mint_amount : nat)"
  dfx --identity pooler1 canister call "$assigned_shard_b_1" mint "($mint_amount : nat)"
  dfx --identity pooler2 canister call "$assigned_shard_2" mint "($mint_amount : nat)"
  dfx --identity pooler2 canister call "$assigned_shard_b_2" mint "($mint_amount : nat)"
  assert_eq "$(dfx canister call --query "$assigned_shard_1" shardBalanceOf "(principal \"$pooler1\")" | grep -oE "$REGEX_NAT")" "$mint_amount : nat"
  assert_eq "$(dfx canister call --query "$assigned_shard_b_1" shardBalanceOf "(principal \"$pooler1\")" | grep -oE "$REGEX_NAT")" "$mint_amount : nat"
  assert_eq "$(dfx canister call --query "$assigned_shard_2" shardBalanceOf "(principal \"$pooler2\")" | grep -oE "$REGEX_NAT")" "$mint_amount : nat"
  assert_eq "$(dfx canister call --query "$assigned_shard_b_2" shardBalanceOf "(principal \"$pooler2\")" | grep -oE "$REGEX_NAT")" "$mint_amount : nat"
fi
end

start "deposit funds on LP"
worker_principal=$(dfx canister id enoki_liquidity_pool_worker)
deposit_shard_a=$(dfx canister call enoki_liquidity_pool_worker getAssignedShardA | grep -oE $REGEX_PRINCIPAL)
deposit_shard_b=$(dfx canister call enoki_liquidity_pool_worker getAssignedShardB | grep -oE $REGEX_PRINCIPAL)
dfx canister call enoki_liquidity_pool_worker register "(principal \"$pooler1\")"
dfx canister call enoki_liquidity_pool_worker register "(principal \"$pooler2\")"
dfx --identity pooler1 canister call "$assigned_shard_1" shardTransferAndCall "(principal \"$deposit_shard_a\", principal \"$worker_principal\", 700_000_000 : nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx --identity pooler1 canister call "$assigned_shard_b_1" shardTransferAndCall "(principal \"$deposit_shard_b\", principal \"$worker_principal\", 700_000_000: nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx --identity pooler2 canister call "$assigned_shard_2" shardTransferAndCall "(principal \"$deposit_shard_a\", principal \"$worker_principal\", 1_000_000_000: nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
info "pooler 1 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler1\")")"
info "pooler 2 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler2\")")"
end