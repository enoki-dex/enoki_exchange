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
to add some test liquidity:
```bash
make liquidity
```
to build the frontend:
```bash
npm i
dfx deploy enoki_exchange_assets
```
The app's local URL should be displayed under the name `enoki_exchange_assets`

## Market Maker Bot

The current iteration of Enoki Exchange and the Market Maker Bot mints and uses test tokens.

The Market Maker Bot retrieves pricing data from CoinGecko and uses it to set limit buy and sell orders.

To run the bot:

For mainnet, make sure you have `dfx` installed and simply run:
```bash
  node ./market_maker_bot/index.js --network ic
```

or with docker:
```bash
  docker compose -f .bot.docker-compose.yml up 
```

For local testnet, first follow the above instructions up to and including `Run`.

Then, still in this root directory, run:
```bash
  node ./market_maker_bot/index.js
```

to stop trading, press `ctr-c` once and wait for open orders to be cancelled.

# Pending Features

- move order_matcher and order_history states to distributed big-maps to allow more scaling
- keep history of swaps
- ability to change brokers easily
- simplify LP withdrawals. Currently it might take a few `removeAllLiquidity` calls to fully withdraw all liquidity.
- make the token balances states easier to follow
- bulk transfer liquidity from LP worker to broker canisters to decentralize liquidity location among subnets
- upgrade `swap` calls with insufficient liquidity to market orders on the main exchange? Or just keep it as it is,
  returning an error telling the user to use the "Trade" tab.
