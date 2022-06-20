import {createClient} from "./dfx.js";
import {parsePrincipal, bigIntToFloat, floatToBigInt, parseNat} from "./utils.js";

const MINT_AMOUNT_A = "1_000_000_000_000_000_000";
const MINT_AMOUNT_B = "3_700_000_000_000_000_000";

const PRICE_DECIMALS = 2;

class Exchange {
  constructor(network) {
    this.id = null;
    this.brokerId = null;
    this.shardA = null;
    this.shardB = null;
    this.depositShardA = null;
    this.depositShardB = null;
    this.client = createClient({network});
    this.orders = [];
  }

  async init() {
    this.id = parsePrincipal(await this.client.exec('identity', 'get-principal'));

    // init shards
    this.shardA = parsePrincipal(await this.client.exec('canister', 'call enoki_wrapped_token register', {principal: this.id}));
    this.shardB = parsePrincipal(await this.client.exec('canister', 'call enoki_wrapped_token_b register', {principal: this.id}));

    // init with exchange
    this.brokerId = parsePrincipal(await this.client.exec('canister', 'call enoki_exchange register', {principal: this.id}));
    this.depositShardA = parsePrincipal(await this.client.exec('canister', `call ${this.brokerId} getAssignedShardA`));
    this.depositShardB = parsePrincipal(await this.client.exec('canister', `call ${this.brokerId} getAssignedShardB`));
    await this.client.exec('canister', `call ${this.brokerId} register`, {principal: this.id});

    // cancel existing orders
    await this.heartbeat();
    await this.cancelAllOrders();
    await this.heartbeat();
  }

  async mint() {
    await this.client.exec('canister', `call ${this.shardA} mint`, {nat: MINT_AMOUNT_A});
    await this.client.exec('canister', `call ${this.shardB} mint`, {nat: MINT_AMOUNT_B});
  }

  async getBalance(forTokenA) {
    if (forTokenA) {
      return parseNat(await this.client.exec('canister', `call ${this.shardA} shardBalanceOf`, {principal: this.id}));
    } else {
      return parseNat(await this.client.exec('canister', `call ${this.shardB} shardBalanceOf`, {principal: this.id}));
    }
  }

  // this function is needed only while the Exchange canister is not automatically using the heartbeat function
  async heartbeat() {
    await this.client.exec('canister', 'call enoki_exchange triggerRun');
  }

  async sendLimitOrder({side, quantity, limitPriceInB, allowTaker}) {
    let order = {
      allow_taker: !!allowTaker,
      limit_price_in_b: limitPriceInB
    };
    let [myShard, depositShard] = side === 'buy' ? [this.shardB, this.depositShardB] : [this.shardA, this.depositShardA];
    this.orders.push(
      await this.client.exec('canister',
        `call ${myShard} shardTransferAndCall`,
        {principal: depositShard}, {principal: this.brokerId},
        {nat: quantity}, {principal: this.brokerId},
        {string: "limitOrder"}, {string: JSON.stringify(order).replace(/"/g, String.raw`\"`)})
    );
  }

  async cancelOrder({id}) {
    await this.client.exec('canister', `call ${this.brokerId} cancelOrder`, {raw: id});
  }

  async cancelAllOrders() {
    await this.client.exec('canister', `call ${this.brokerId} cancelAllOpenOrders`);
  }

  async logOff() {
    await this.cancelAllOrders();
    await this.heartbeat();
  }
}

export default Exchange;
