set -x
set -a # automatically export all variables
source .env
set +a

dfx identity use default
dfx canister create --all

./src/enoki_exchange/install.sh
./src/enoki_liquidity_pool/install.sh
./src/enoki_liquidity_pool_worker/install.sh

./src/enoki_broker/install.sh

