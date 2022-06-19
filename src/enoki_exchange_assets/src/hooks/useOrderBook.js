import React from "react";
import getEnokiExchange from "../actors/getEnokiExchange";
import {bigIntToFloat} from "../utils/utils";

const ORDER_BOOK_LENGTH = 7;

const priceToFloat = (priceInt, numDecimals) => {
  return Number(priceInt) / Math.pow(10, Number(numDecimals));
}

const useOrderBook = () => {
  const [bids, setBids] = React.useState([]);
  const [asks, setAsks] = React.useState([]);

  React.useEffect(() => {
    let stop = false;
    const fetch = () => getEnokiExchange(undefined).getBidAskCurve()
      .then(bidAsk => {
        if (stop) return;

        let bids = bidAsk.bids;
        let asks = bidAsk.asks;
        asks.sort((a, b) => {
          if (a[0] < b[0]) {
            return -1;
          } else if (a[0] > b[0]) {
            return 1;
          } else {
            return 0;
          }
        });
        asks = asks.slice(0, ORDER_BOOK_LENGTH);
        asks.reverse();
        bids.sort((a, b) => {
          if (a[0] < b[0]) {
            return 1;
          } else if (a[0] > b[0]) {
            return -1;
          } else {
            return 0;
          }
        });
        bids = bids.slice(0, ORDER_BOOK_LENGTH);

        bids = bids.map(([price, amount]) => [priceToFloat(price, bidAsk.num_decimals), bigIntToFloat(amount, 'eICP')]);
        asks = asks.map(([price, amount]) => [priceToFloat(price, bidAsk.num_decimals), bigIntToFloat(amount, 'eICP')]);
        setBids(bids);
        setAsks(asks);
      })
      .catch(err => console.error("error retrieving order book: ", err));

    const wait = delay => new Promise(resolve => setTimeout(resolve, delay));
    const run = async () => {
      while (!stop) {
        await fetch();
        await wait(5000);
      }
    }

    let _ = run();

    return () => {
      stop = true;
    }
  }, []);

  return {
    bids, asks
  };
}

export default useOrderBook;
