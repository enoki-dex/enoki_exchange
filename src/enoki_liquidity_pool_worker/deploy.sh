set -x
. "$(dirname "$0")"/build.sh
MANAGER_ID="principal \"$(
  dfx canister id enoki_liquidity_pool
)\""

dfx deploy enoki_liquidity_pool_worker
dfx canister call enoki_liquidity_pool_worker finishInit "($MANAGER_ID)"
dfx canister call enoki_liquidity_pool initWorker "(principal \"$(dfx canister id enoki_liquidity_pool_worker)\")"
dfx canister call enoki_exchange initPool "($MANAGER_ID)"
