echo "deploying for token id: $APP_ACCEPTED_TOKEN"
. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
dfx build enoki_liquidity_pool
OWNER="principal \"$( \
   dfx identity get-principal
)\""
yes yes | dfx canister install enoki_liquidity_pool --argument "($APP_ACCEPTED_TOKEN, $OWNER)" -m=reinstall
