set -a # automatically export all variables
source .env
set +a

REGEX_PRINCIPAL='(?:[a-z0-9]+\-[a-z0-9]+)+'
REGEX_NAT='[0-9_]+ : nat'

if [ -z "$APP_TOKEN_A" ]; then
  APP_TOKEN_A=$(dfx canister id enoki_wrapped_token)
  export APP_TOKEN_A
  APP_TOKEN_B=$(dfx canister id enoki_wrapped_token_b)
  export APP_TOKEN_B
  echo "setting tokens A=$APP_TOKEN_A B=$APP_TOKEN_B"
else
  echo "tokens already set A=$APP_TOKEN_A B=$APP_TOKEN_B"
fi

dfx identity new pooler1 || true
dfx identity new pooler2 || true
pooler1=$(dfx --identity pooler1 identity get-principal)
echo "pooler1: $pooler1"
pooler2=$(dfx --identity pooler2 identity get-principal)
echo "pooler2: $pooler2"
dfx identity new trader1 || true
dfx identity new trader2 || true
trader1=$(dfx --identity trader1 identity get-principal)
echo "trader1: $trader1"
trader2=$(dfx --identity trader2 identity get-principal)
echo "trader2: $trader2"

dfx identity use default
mint_amount="1_000_000_000_000_000_000"
assigned_shard_1=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$pooler1\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_1=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$pooler1\")" | grep -oE "$REGEX_PRINCIPAL")
echo "pooler1 assigned shards: $assigned_shard_1 / $assigned_shard_b_1"
assigned_shard_2=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$pooler2\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_2=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$pooler2\")" | grep -oE "$REGEX_PRINCIPAL")
echo "pooler2 assigned shards: $assigned_shard_2 / $assigned_shard_b_2"
dfx --identity pooler1 canister call "$assigned_shard_1" mint "($mint_amount : nat)"
dfx --identity pooler1 canister call "$assigned_shard_b_1" mint "($mint_amount : nat)"
dfx --identity pooler2 canister call "$assigned_shard_2" mint "($mint_amount : nat)"
dfx --identity pooler2 canister call "$assigned_shard_b_2" mint "($mint_amount : nat)"

worker_principal=$(dfx canister id enoki_liquidity_pool_worker)
deposit_shard_a=$(dfx canister call enoki_liquidity_pool_worker getAssignedShardA | grep -oE $REGEX_PRINCIPAL)
deposit_shard_b=$(dfx canister call enoki_liquidity_pool_worker getAssignedShardB | grep -oE $REGEX_PRINCIPAL)
dfx canister call enoki_liquidity_pool_worker register "(principal \"$pooler1\")"
dfx canister call enoki_liquidity_pool_worker register "(principal \"$pooler2\")"
dfx --identity pooler1 canister call "$assigned_shard_1" shardTransferAndCall "(principal \"$deposit_shard_a\", principal \"$worker_principal\", $mint_amount : nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx --identity pooler1 canister call "$assigned_shard_b_1" shardTransferAndCall "(principal \"$deposit_shard_b\", principal \"$worker_principal\", $mint_amount : nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx --identity pooler2 canister call "$assigned_shard_2" shardTransferAndCall "(principal \"$deposit_shard_a\", principal \"$worker_principal\", $mint_amount : nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx --identity pooler2 canister call "$assigned_shard_b_2" shardTransferAndCall "(principal \"$deposit_shard_b\", principal \"$worker_principal\", $mint_amount: nat, principal \"$worker_principal\", \"addLiquidity\", \"\")"
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
echo "pooler 1 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler1\")")"
echo "pooler 2 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler2\")")"

assigned_shard_1=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$trader1\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_1=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$trader1\")" | grep -oE "$REGEX_PRINCIPAL")
echo "trader1 assigned shards: $assigned_shard_1 / $assigned_shard_b_1"
assigned_shard_2=$(dfx canister call "$APP_TOKEN_A" register "(principal \"$trader2\")" | grep -oE "$REGEX_PRINCIPAL")
assigned_shard_b_2=$(dfx canister call "$APP_TOKEN_B" register "(principal \"$trader2\")" | grep -oE "$REGEX_PRINCIPAL")
echo "trader2 assigned shards: $assigned_shard_2 / $assigned_shard_b_2"
dfx --identity trader1 canister call "$assigned_shard_1" mint "($mint_amount : nat)"
dfx --identity trader1 canister call "$assigned_shard_b_1" mint "($mint_amount : nat)"
dfx --identity trader2 canister call "$assigned_shard_2" mint "($mint_amount : nat)"
dfx --identity trader2 canister call "$assigned_shard_b_2" mint "($mint_amount : nat)"

broker_1=$(dfx --identity trader1 canister call enoki_exchange register "(principal \"$trader1\")" | grep -oE "$REGEX_PRINCIPAL")
broker_2=$(dfx --identity trader2 canister call enoki_exchange register "(principal \"$trader2\")" | grep -oE "$REGEX_PRINCIPAL")

deposit_shard_a_1=$(dfx canister call "$broker_1" getAssignedShardA | grep -oE $REGEX_PRINCIPAL)
deposit_shard_b_1=$(dfx canister call "$broker_1" getAssignedShardB | grep -oE $REGEX_PRINCIPAL)
deposit_shard_a_2=$(dfx canister call "$broker_2" getAssignedShardA | grep -oE $REGEX_PRINCIPAL)
deposit_shard_b_2=$(dfx canister call "$broker_2" getAssignedShardB | grep -oE $REGEX_PRINCIPAL)

dfx canister call "$broker_1" register "(principal \"$trader1\")"
dfx canister call "$broker_2" register "(principal \"$trader2\")"

dfx --identity trader1 canister call "$assigned_shard_1" shardTransferAndCall "(principal \"$deposit_shard_a_1\", principal \"$broker_1\", 300_000_000_000_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 6.0}\")"
dfx --identity trader2 canister call "$assigned_shard_2" shardTransferAndCall "(principal \"$deposit_shard_a_2\", principal \"$broker_2\", 200_000_000_000_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.95}\")"
dfx --identity trader1 canister call "$assigned_shard_1" shardTransferAndCall "(principal \"$deposit_shard_a_1\", principal \"$broker_1\", 300_000_000_000_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 6.01}\")"
dfx --identity trader2 canister call "$assigned_shard_2" shardTransferAndCall "(principal \"$deposit_shard_a_2\", principal \"$broker_2\", 200_000_000_000_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.97}\")"
dfx --identity trader1 canister call "$assigned_shard_1" shardTransferAndCall "(principal \"$deposit_shard_a_1\", principal \"$broker_1\", 200_000_000_000_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 6.02}\")"
dfx --identity trader2 canister call "$assigned_shard_2" shardTransferAndCall "(principal \"$deposit_shard_a_2\", principal \"$broker_2\", 300_000_000_000_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.98}\")"
dfx --identity trader1 canister call "$assigned_shard_1" shardTransferAndCall "(principal \"$deposit_shard_a_1\", principal \"$broker_1\", 200_000_000_000_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 6.02}\")"
dfx --identity trader2 canister call "$assigned_shard_2" shardTransferAndCall "(principal \"$deposit_shard_a_2\", principal \"$broker_2\", 300_000_000_000_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 6.03}\")"

dfx --identity trader1 canister call "$assigned_shard_b_1" shardTransferAndCall "(principal \"$deposit_shard_b_1\", principal \"$broker_1\", 300_000_000_000_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.93}\")"
dfx --identity trader2 canister call "$assigned_shard_b_2" shardTransferAndCall "(principal \"$deposit_shard_b_2\", principal \"$broker_2\", 300_000_000_000_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.92}\")"
dfx --identity trader1 canister call "$assigned_shard_b_1" shardTransferAndCall "(principal \"$deposit_shard_b_1\", principal \"$broker_1\", 300_000_000_000_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.91}\")"
dfx --identity trader2 canister call "$assigned_shard_b_2" shardTransferAndCall "(principal \"$deposit_shard_b_2\", principal \"$broker_2\", 300_000_000_000_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.90}\")"
dfx --identity trader1 canister call "$assigned_shard_b_1" shardTransferAndCall "(principal \"$deposit_shard_b_1\", principal \"$broker_1\", 200_000_000_000_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.88}\")"
dfx --identity trader2 canister call "$assigned_shard_b_2" shardTransferAndCall "(principal \"$deposit_shard_b_2\", principal \"$broker_2\", 200_000_000_000_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.87}\")"
dfx --identity trader1 canister call "$assigned_shard_b_1" shardTransferAndCall "(principal \"$deposit_shard_b_1\", principal \"$broker_1\", 200_000_000_000_000_000 : nat, principal \"$broker_1\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.86}\")"
dfx --identity trader2 canister call "$assigned_shard_b_2" shardTransferAndCall "(principal \"$deposit_shard_b_2\", principal \"$broker_2\", 200_000_000_000_000_000 : nat, principal \"$broker_2\", \"limitOrder\", \"{\\\"allow_taker\\\": false, \\\"limit_price_in_b\\\": 5.85}\")"

dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat

dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
