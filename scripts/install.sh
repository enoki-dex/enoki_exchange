set -a # automatically export all variables
source .env
set +a

dfx identity use default
#./src/enoki_exchange/deploy.sh
./src/enoki_liquidity_pool/deploy.sh

