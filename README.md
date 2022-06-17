# Enoki Exchange

Enoki Exchange is a fully asynchronous and scalable DEX built on the Internet Computer.

It uses "broker" smart contract canisters to take orders from users. The exchange 
routinely syncs with the brokers to execute limit orders and create a bid/ask curve that is used to
execute swaps.

Swaps are executed directly by the brokers using liquidity from a liquidity pool and with prices obtained from the
bid/ask curve. Part of the fee goes to the LP providers, and part goes to the market makers who set the limit orders.

# Development

## Dependencies

- [rust](https://rustup.rs)
- [dfx](https://smartcontracts.org/docs/developers-guide/install-upgrade-remove.html)
- [cmake](https://cmake.org/)

[//]: # (- [npm]&#40;https://nodejs.org/en/download/&#41;)

Make sure you have wasm as a target:
```
rustup target add wasm32-unknown-unknown
```

## Local Deploy

### Configure

```bash
cp default.env .env
```

### Run
make sure to use `make deploy` below (instead of simply `dfx deploy`) to initialize the canisters with parameters from `.env`
```bash
dfx start --background
make deps
make deploy
```

### Test
after running the above commands:
```bash
make test
```

### Local Frontend
to build the frontend:
```bash
dfx deploy
```
The app's local URL should be displayed.

# Pending Features

- move order_matcher and order_history states to distributed big-maps to allow more scaling
- keep history of swaps
- ability to change brokers easily
- simplify LP withdrawals. Currently it might take a few `removeAllLiquidity` calls to fully withdraw all liquidity.
- make the token balances states easier to follow
- bulk transfer liquidity from LP worker to broker canisters to decentralize liquidity location among subnets
- upgrade `swap` calls with insufficient liquidity to market orders on the main exchange? Or just keep it as it is,
  returning an error telling the user to use the "Trade" tab.
