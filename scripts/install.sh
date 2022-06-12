set -x
set -a # automatically export all variables
source .env
set +a

if [ -z "$APP_TOKEN_A" ]; then
  APP_TOKEN_A=$(dfx canister id enoki_wrapped_token)
  export APP_TOKEN_A
  APP_TOKEN_B=$(dfx canister id enoki_wrapped_token_b)
  export APP_TOKEN_B
    echo "setting tokens A=$APP_TOKEN_A B=$APP_TOKEN_B";
  else
    echo "tokens already set A=$APP_TOKEN_A B=$APP_TOKEN_B";
fi

dfx identity use default
dfx canister create --all

./src/enoki_exchange/install.sh
./src/enoki_liquidity_pool/install.sh
./src/enoki_liquidity_pool_worker/install.sh

./src/enoki_broker/install.sh
