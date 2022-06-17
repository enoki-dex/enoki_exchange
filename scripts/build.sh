set -x

./src/enoki_exchange/build.sh
./src/enoki_liquidity_pool/build.sh
./src/enoki_liquidity_pool_worker/build.sh

./src/enoki_broker/build.sh
