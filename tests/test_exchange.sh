. "$(dirname "$0")"/setup.sh

if [ -z "$APP_TOKEN_A" ]; then
  APP_TOKEN_A=$(dfx canister id enoki_wrapped_token)
  export APP_TOKEN_A
  APP_TOKEN_B=$(dfx canister id enoki_wrapped_token_b)
  export APP_TOKEN_B
  echo "setting tokens A=$APP_TOKEN_A B=$APP_TOKEN_B"
else
  echo "tokens already set A=$APP_TOKEN_A B=$APP_TOKEN_B"
fi

start "creating users"
dfx identity new trader1 || true
dfx identity new trader2 || true
dfx identity new swapper1 || true
dfx identity new swapper2 || true
trader1=$(dfx --identity trader1 identity get-principal)
info "trader1: $trader1"
trader2=$(dfx --identity trader2 identity get-principal)
info "trader2: $trader2"
swapper1=$(dfx --identity swapper1 identity get-principal)
info "swapper1: $swapper1"
swapper2=$(dfx --identity swapper2 identity get-principal)
info "swapper2: $swapper2"
end

start "fund users"
dfx identity use default
mint_amount="1_000_000_000"
assigned_shard_1=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$trader1\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_1=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$trader1\")" | grep -oE "$REGEX_PRINCIPAL")
info "trader1 assigned shards: $assigned_shard_1 / $assigned_shard_b_1"
assigned_shard_2=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$trader2\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_2=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$trader2\")" | grep -oE "$REGEX_PRINCIPAL")
info "trader2 assigned shards: $assigned_shard_2 / $assigned_shard_b_2"
assigned_shard_3=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$swapper1\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_3=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$swapper1\")" | grep -oE "$REGEX_PRINCIPAL")
info "swapper1 assigned shards: $assigned_shard_3 / $assigned_shard_b_3"
assigned_shard_4=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$swapper2\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_4=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$swapper2\")" | grep -oE "$REGEX_PRINCIPAL")
info "swapper2 assigned shards: $assigned_shard_4 / $assigned_shard_b_4"
if [ "0 : nat" == "$(dfx canister call "$assigned_shard_3" shardBalanceOf "(principal \"$swapper1\")" | grep -oE "$REGEX_NAT")" ]; then
  dfx --identity trader1 canister call "$assigned_shard_1" mint "($mint_amount : nat)"
  dfx --identity trader1 canister call "$assigned_shard_b_1" mint "($mint_amount : nat)"
  dfx --identity trader2 canister call "$assigned_shard_2" mint "($mint_amount : nat)"
  dfx --identity trader2 canister call "$assigned_shard_b_2" mint "($mint_amount : nat)"
  dfx --identity swapper1 canister call "$assigned_shard_3" mint "($mint_amount : nat)"
  dfx --identity swapper1 canister call "$assigned_shard_b_3" mint "($mint_amount : nat)"
  dfx --identity swapper2 canister call "$assigned_shard_4" mint "($mint_amount : nat)"
  dfx --identity swapper2 canister call "$assigned_shard_b_4" mint "($mint_amount : nat)"
fi
end

start "submit limit trades"
broker_1=$(dfx --identity trader1 canister call enoki_exchange register "(principal \"$trader1\")" | grep -oE "$REGEX_PRINCIPAL")
broker_2=$(dfx --identity trader2 canister call enoki_exchange register "(principal \"$trader2\")" | grep -oE "$REGEX_PRINCIPAL")
broker_3=$(dfx --identity swapper1 canister call enoki_exchange register "(principal \"$swapper1\")" | grep -oE "$REGEX_PRINCIPAL")
broker_4=$(dfx --identity swapper2 canister call enoki_exchange register "(principal \"$swapper2\")" | grep -oE "$REGEX_PRINCIPAL")

deposit_shard_a_1=$(dfx canister call "$broker_1" getAssignedShardA | grep -oE $REGEX_PRINCIPAL)
deposit_shard_b_1=$(dfx canister call "$broker_1" getAssignedShardB | grep -oE $REGEX_PRINCIPAL)
deposit_shard_a_2=$(dfx canister call "$broker_2" getAssignedShardA | grep -oE $REGEX_PRINCIPAL)
deposit_shard_b_2=$(dfx canister call "$broker_2" getAssignedShardB | grep -oE $REGEX_PRINCIPAL)
deposit_shard_a_3=$(dfx canister call "$broker_3" getAssignedShardA | grep -oE $REGEX_PRINCIPAL)
deposit_shard_b_3=$(dfx canister call "$broker_3" getAssignedShardB | grep -oE $REGEX_PRINCIPAL)
deposit_shard_a_4=$(dfx canister call "$broker_4" getAssignedShardA | grep -oE $REGEX_PRINCIPAL)
deposit_shard_b_4=$(dfx canister call "$broker_4" getAssignedShardB | grep -oE $REGEX_PRINCIPAL)

dfx canister call "$broker_1" register "(principal \"$trader1\")"
dfx canister call "$broker_2" register "(principal \"$trader2\")"
dfx canister call "$broker_3" register "(principal \"$swapper1\")"
dfx canister call "$broker_4" register "(principal \"$swapper2\")"

dfx --identity trader1 canister call "$assigned_shard_1" shardTransferAndCall "(principal \"$deposit_shard_a_1\", principal \"$broker_1\", 400_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 6.0}\")"
dfx --identity trader2 canister call "$assigned_shard_2" shardTransferAndCall "(principal \"$deposit_shard_a_2\", principal \"$broker_2\", 300_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.95}\")"
dfx --identity trader2 canister call "$assigned_shard_b_2" shardTransferAndCall "(principal \"$deposit_shard_b_2\", principal \"$broker_2\", 300_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.90}\")"

dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat

info "swapper1 before swap balance A: $(dfx canister call "$APP_TOKEN_A" balanceOf "(principal \"$swapper1\")"), B: $(dfx canister call "$APP_TOKEN_B" balanceOf "(principal \"$swapper1\")")"
dfx --identity swapper1 canister call "$assigned_shard_3" shardTransferAndCall "(principal \"$deposit_shard_a_3\", principal \"$broker_3\", 50_000_000 : nat, principal \"$broker_3\", \"swap\", \"{\\\"allow_taker\\\": true, \\\"limit_price_in_b\\\": 5.85}\")"
info "swapper1 after swap 1 balance A: $(dfx canister call "$APP_TOKEN_A" balanceOf "(principal \"$swapper1\")"), B: $(dfx canister call "$APP_TOKEN_B" balanceOf "(principal \"$swapper1\")")"
dfx --identity swapper1 canister call "$assigned_shard_b_3" shardTransferAndCall "(principal \"$deposit_shard_b_3\", principal \"$broker_3\", 50_000_000 : nat, principal \"$broker_3\", \"swap\", \"{\\\"allow_taker\\\": true, \\\"limit_price_in_b\\\": 6.1}\")"
info "swapper1 after swap 2 balance A: $(dfx canister call "$APP_TOKEN_A" balanceOf "(principal \"$swapper1\")"), B: $(dfx canister call "$APP_TOKEN_B" balanceOf "(principal \"$swapper1\")")"

dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat

dfx --identity trader1 canister call "$assigned_shard_b_1" shardTransferAndCall "(principal \"$deposit_shard_b_1\", principal \"$broker_1\", 400_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": true, \\\"limit_price_in_b\\\": 5.96}\")"

info "swapper2 before swap balance A: $(dfx canister call "$APP_TOKEN_A" balanceOf "(principal \"$swapper2\")"), B: $(dfx canister call "$APP_TOKEN_B" balanceOf "(principal \"$swapper2\")")"
dfx --identity swapper2 canister call "$assigned_shard_4" shardTransferAndCall "(principal \"$deposit_shard_a_4\", principal \"$broker_4\", 500_000 : nat, principal \"$broker_4\", \"swap\", \"{\\\"allow_taker\\\": true, \\\"limit_price_in_b\\\": 5.85}\")"
info "swapper2 after swap 1 balance A: $(dfx canister call "$APP_TOKEN_A" balanceOf "(principal \"$swapper2\")"), B: $(dfx canister call "$APP_TOKEN_B" balanceOf "(principal \"$swapper2\")")"
dfx --identity swapper2 canister call "$assigned_shard_b_4" shardTransferAndCall "(principal \"$deposit_shard_b_4\", principal \"$broker_4\", 500_000 : nat, principal \"$broker_4\", \"swap\", \"{\\\"allow_taker\\\": true, \\\"limit_price_in_b\\\": 6.1}\")"
info "swapper2 after swap 2 balance A: $(dfx canister call "$APP_TOKEN_A" balanceOf "(principal \"$swapper2\")"), B: $(dfx canister call "$APP_TOKEN_B" balanceOf "(principal \"$swapper2\")")"

dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
end
