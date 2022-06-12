set -x
. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
dfx canister create enoki_liquidity_pool
dfx canister create enoki_liquidity_pool_worker
dfx build enoki_liquidity_pool_worker
MANAGER_ID="principal \"$(
  dfx canister id enoki_liquidity_pool
)\""

yes yes | dfx canister install enoki_liquidity_pool_worker
dfx canister call enoki_liquidity_pool_worker finishInit "($MANAGER_ID)"
dfx canister call enoki_liquidity_pool "initWorker" "(principal \"$(dfx canister id enoki_liquidity_pool_worker)\")"
dfx canister call enoki_exchange "initPool" "($MANAGER_ID)"
