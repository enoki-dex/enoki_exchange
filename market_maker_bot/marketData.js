import https from 'https';

//TODO: change your price provider here
// Using CoinGecko's free api, which is rate limited to ~50 requests / sec.

class MarketData {
  constructor() {
    this.apiHost = "api.coingecko.com";
    this.basePath = "/api/v3";
    this.coinId = "internet-computer";
    this.pricingCurrency = "usd";
    this.sdrToUsd = 1.42;
  }

  get(path) {
    const options = {
      hostname: this.apiHost,
      port: 443,
      path: `${this.basePath}/${path}`,
      method: 'GET',
    };

    return new Promise((resolve, reject) => {
      const req = https.request(options, res => {
        let response = '';
        res.on('data', chunk => {
          response += chunk;
        });
        res.on('end', () => {
          resolve(response);
        })
      });

      req.on('error', error => {
        reject(error);
      });

      req.end();
    });
  }

  async getLatestPrice() {
    // request:
    // simple/price?ids=internet-computer&vs_currencies=usd

    let response = JSON.parse(await this.get(`simple/price?ids=${this.coinId}&vs_currencies=${this.pricingCurrency}`));

    // response:
    // {
    //   "internet-computer": {
    //     "usd": 5.33
    //   }
    // }

    let priceUsd = parseFloat(response[this.coinId][this.pricingCurrency])
    return priceUsd / this.sdrToUsd;
  }
}

export default MarketData;
