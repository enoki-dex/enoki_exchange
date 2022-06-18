set -x
. "$(dirname "$0")"/build.sh
MANAGER_ID="principal \"$(
  dfx canister id enoki_exchange
)\""
CANISTER_NAME="enoki_broker"

i=1
num_brokers=${NUMBER_OF_BROKERS:-2}
while [ $i -le "$num_brokers" ]; do
  dfx deploy "${CANISTER_NAME}_$i"
  dfx canister call "${CANISTER_NAME}_$i" finishInit "($MANAGER_ID)"
  dfx canister call enoki_exchange addBroker "(principal \"$(dfx canister id "${CANISTER_NAME}_$i")\")"
  dfx canister call "${CANISTER_NAME}_$i" updateUpstreamFees
  true $((i++))
done
