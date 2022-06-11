. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
dfx build enoki_liquidity_pool
OWNER="principal \"$(
  dfx identity get-principal
)\""
MANAGER_ID="principal \"$(
  dfx canister id enoki_exchange
)\""

yes yes | dfx canister install enoki_liquidity_pool --argument "($OWNER, $MANAGER_ID)" -m=reinstall
