echo "deploying exchange for token pair: $APP_TOKEN_A / $APP_TOKEN_B"
. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
dfx build enoki_exchange
OWNER="principal \"$(
  dfx identity get-principal
)\""

yes yes | dfx canister install enoki_exchange --argument "($OWNER, principal \"$APP_TOKEN_A\", principal \"$APP_TOKEN_B\", $PRICE_NUMBER_OF_DECIMALS)" -m=reinstall
