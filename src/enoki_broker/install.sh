set -x
. "$(dirname "$0")"/build.sh
#ic-cdk-optimizer "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/enoki_wrapped_token.wasm -o "$(dirname "$0")"../../target/wasm32-unknown-unknown/release/opt.wasm
MANAGER_ID="principal \"$(
  dfx canister id enoki_exchange
)\""
CANISTER_NAME="enoki_broker"

i=1
num_brokers=${NUMBER_OF_BROKERS:-2}
while [ $i -le "$num_brokers" ]; do
  dfx build "${CANISTER_NAME}_$i"
  yes yes | dfx canister install "${CANISTER_NAME}_$i" -m=reinstall
  dfx canister call "${CANISTER_NAME}_$i" finishInit "($MANAGER_ID)"
  dfx canister call enoki_exchange "addBroker" "(principal \"$(dfx canister id "${CANISTER_NAME}_$i")\")"
  true $((i++))
done