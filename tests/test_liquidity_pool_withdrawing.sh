. "$(dirname "$0")"/setup.sh

if [ -z "$APP_TOKEN_A" ]; then
  APP_TOKEN_A=$(dfx canister id enoki_wrapped_token)
  export APP_TOKEN_A
  APP_TOKEN_B=$(dfx canister id enoki_wrapped_token_b)
  export APP_TOKEN_B
  echo "setting tokens A=$APP_TOKEN_A B=$APP_TOKEN_B"
else
  echo "tokens already set A=$APP_TOKEN_A B=$APP_TOKEN_B"
fi

start "creating users"
dfx identity new pooler1 || true
dfx identity new pooler2 || true
pooler1=$(dfx --identity pooler1 identity get-principal)
info "pooler1: $pooler1"
pooler2=$(dfx --identity pooler2 identity get-principal)
info "pooler2: $pooler2"
dfx identity use default
end

start "withdraw funds from LP"
info "pooler 1 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler1\")")"
info "pooler 2 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler2\")")"
info "before withdraw balance A: $(dfx canister call "$APP_TOKEN_A" balanceOf "(principal \"$pooler2\")"), B: $(dfx canister call "$APP_TOKEN_B" balanceOf "(principal \"$pooler2\")")"
dfx --identity pooler2 canister call enoki_liquidity_pool_worker removeAllLiquidity

dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat
dfx canister call enoki_exchange triggerRun
dfx canister call enoki_liquidity_pool_worker triggerHeartbeat

info "pooler 2 liquidity: $(dfx canister call enoki_liquidity_pool_worker getLiquidity "(principal \"$pooler2\")")"
info "after withdraw balance A: $(dfx canister call "$APP_TOKEN_A" balanceOf "(principal \"$pooler2\")"), B: $(dfx canister call "$APP_TOKEN_B" balanceOf "(principal \"$pooler2\")")"
end
