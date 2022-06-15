echo "deploying exchange for token pair: $APP_TOKEN_A / $APP_TOKEN_B"
. "$(dirname "$0")"/build.sh

dfx deploy enoki_exchange
dfx canister call enoki_exchange finishInit "(principal \"$APP_TOKEN_A\", principal \"$APP_TOKEN_B\", $PRICE_NUMBER_OF_DECIMALS)"
dfx canister call enoki_exchange setFees "($DEPOSIT_FEE_TOKEN_A : nat, $DEPOSIT_FEE_TOKEN_B: nat, $LIMIT_ORDER_TAKER_FEE, $SWAP_FEE, $SWAP_MARKET_MAKER_REWARD)"
