import Exchange from "./exchange.js";
import {setTimeout} from 'timers/promises';
import MarketData from "./marketData.js";

const TICK_INTERVAL_MS = 3_000;
const BID_ASK_SPREAD_EACH_IN_CENTS = 2;
const FRACTION_OF_BALANCE_TO_SPEND = 20;

class App {
  constructor(network) {
    this.network = network;
    this.exchange = new Exchange(network);
    this.exitCalled = false;
    this.marketData = new MarketData();
    this.currentPriceCents = 0;
  }

  async mint() {
    await this.exchange.init();
    await this.exchange.mint();
  }

  async run() {
    await this.exchange.init();
    console.log('[app] init done');

    while (!this.exitCalled) {
      try {
        await this.checkPrice();
      } catch (err) {
        console.error("[app] ERROR trading: ", err);
      }
      await setTimeout(TICK_INTERVAL_MS);
    }

    await this.exchange.logOff();
  }

  async checkPrice() {
    let price = Math.round(await this.marketData.getLatestPrice() * 100);
    if (price !== this.currentPriceCents) {
      console.log(`[app] price changed from ${this.currentPriceCents} to ${price}`);
      this.currentPriceCents = price;
      await this.trade();
    }
  }

  async trade() {
    await this.exchange.cancelAllOrders();

    let bidPrice = (this.currentPriceCents - BID_ASK_SPREAD_EACH_IN_CENTS) / 100;
    let askPrice = (this.currentPriceCents + BID_ASK_SPREAD_EACH_IN_CENTS) / 100;
    let [bidAmount, askAmount] = (await Promise.all([
      this.exchange.getBalance(false),
      this.exchange.getBalance(true),
    ])).map(balance => balance / BigInt(FRACTION_OF_BALANCE_TO_SPEND));

    // send only one order:
    // await this.exchange.sendLimitOrder({side: 'buy', quantity: bidAmount, limitPriceInB: bidPrice, allowTaker: true});
    // await this.exchange.sendLimitOrder({side: 'sell', quantity: askAmount, limitPriceInB: askPrice, allowTaker: true});

    // or multiple:
    let diffs = [0, 0.01, 0.02, 0.03, 0.04, 0.05, 0.06];
    let bids = diffs.map(diff => bidPrice - diff);
    let asks = diffs.map(diff => askPrice + diff);

    await Promise.all([
      bids.map(bid => this.exchange.sendLimitOrder({
        side: 'buy',
        quantity: bidAmount,
        limitPriceInB: bid,
        allowTaker: true
      })),
      asks.map(ask => this.exchange.sendLimitOrder({
        side: 'sell',
        quantity: askAmount,
        limitPriceInB: ask,
        allowTaker: true
      }))
    ].flatMap(m => m));


    await this.exchange.heartbeat();
  }

  exit() {
    this.exitCalled = true;
  }
}

export default App;
