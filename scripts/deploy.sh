set -x
set -a # automatically export all variables
source .env
set +a

dfx identity use default
dfx canister create --all

./src/enoki_exchange/deploy.sh
./src/enoki_liquidity_pool/deploy.sh
./src/enoki_liquidity_pool_worker/deploy.sh

./src/enoki_broker/deploy.sh

