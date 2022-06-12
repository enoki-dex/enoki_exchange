. "$(dirname "$0")"/setup.sh

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
if [ "0 : nat" == "$(dfx canister call "$assigned_shard_1" shardBalanceOf "(principal \"$pooler1\")" | grep -oE "$REGEX_NAT")" ]; then
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
dfx --identity pooler1 canister call "$assigned_shard_1" shardTransferAndCall "(principal \"$deposit_shard_a\", principal \"$worker_principal\", 700_000_000 : nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx --identity pooler1 canister call "$assigned_shard_b_1" shardTransferAndCall "(principal \"$deposit_shard_b\", principal \"$worker_principal\", 700_000_000: nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx --identity pooler2 canister call "$assigned_shard_2" shardTransferAndCall "(principal \"$deposit_shard_a\", principal \"$worker_principal\", 1_000_000_000: nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
info "pooler 1 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler1\")")"
info "pooler 2 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler2\")")"
end

#start "setting up user1 on enoki_wrapped_token"
#dfx identity use user1
#ASSIGNED_SHARD=$(dfx canister call enoki_wrapped_token register "(principal \"$USER1\")" | grep -oE $REGEX_PRINCIPAL)
#info "user1 assigned to: $ASSIGNED_SHARD"
#dfx canister call xtc_token approve "(principal \"$ASSIGNED_SHARD\", 12300000000)"
#info "wrapping original token"
#dfx canister call "$ASSIGNED_SHARD" wrap "(12300000000)"
#info "user1 balance original token: $(dfx canister call xtc_token balanceOf "(principal \"$USER1\")")"
#info "user1 balance of wrapped token: $(dfx canister call enoki_wrapped_token balanceOf "(principal \"$USER1\")")"
#info "total supply of wrapped token: $(dfx canister call enoki_wrapped_token totalSupply)"
#end
#
#start "deposit to exchange"
#DEPOSIT_SHARD=$(dfx canister call mock_exchange getDepositShardId | grep -oE $REGEX_PRINCIPAL)
#EXCHANGE_ID=$(dfx canister id mock_exchange)
#dfx canister call "$ASSIGNED_SHARD" shardTransferAndCall "(principal \"$DEPOSIT_SHARD\", principal \"$EXCHANGE_ID\", 1220000000, principal \"$EXCHANGE_ID\", \"deposit\")"
#BALANCE=$(dfx canister call mock_exchange balance)
#info "user1 balance on exchange: $BALANCE"
#assert_eq "$BALANCE" "(1_219_980_000 : nat)"
#info "user1 balance of wrapped token: $(dfx canister call enoki_wrapped_token balanceOf "(principal \"$USER1\")")"
#info "total supply of wrapped token: $(dfx canister call enoki_wrapped_token totalSupply)"
#info "total accrued fees of wrapped token: $(dfx canister call enoki_wrapped_token getAccruedFees)"
#end
#
#start "unwrap token"
#info "withdrawing from exchange"
#dfx canister call mock_exchange withdrawAll "(principal \"$ASSIGNED_SHARD\", principal \"$USER1\")"
#info "user1 balance on exchange: $(dfx canister call mock_exchange balance)"
#info "user1 balance of wrapped token: $(dfx canister call enoki_wrapped_token balanceOf "(principal \"$USER1\")")"
#AMOUNT="11_079_999_580"
#info "unwrapping $AMOUNT tokens"
#dfx canister call "$ASSIGNED_SHARD" unwrap "($AMOUNT, principal \"$USER1\")"
#info "user1 balance original token: $(dfx canister call xtc_token balanceOf "(principal \"$USER1\")")"
#info "user1 balance of wrapped token: $(dfx canister call enoki_wrapped_token balanceOf "(principal \"$USER1\")")"
#info "total supply of wrapped token: $(dfx canister call enoki_wrapped_token totalSupply)"
#info "total accrued fees of wrapped token: $(dfx canister call enoki_wrapped_token getAccruedFees)"
#end
