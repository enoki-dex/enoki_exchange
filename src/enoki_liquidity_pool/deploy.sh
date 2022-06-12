. "$(dirname "$0")"/build.sh
MANAGER_ID="principal \"$(
  dfx canister id enoki_exchange
)\""

dfx deploy enoki_liquidity_pool
dfx canister call enoki_liquidity_pool finishInit "($MANAGER_ID)"
