import React from "react";
import useLogin from "../../hooks/useLogin";
import {useSelector, useDispatch} from 'react-redux'
import {setAllowTaker, setOnlyMaker} from "../../state/tradeSlice";
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import {bigIntToStr, floatToBigInt} from "../../utils/utils";
import {getAssignedTokenShard} from "../../actors/getMainToken";
import getEnokiExchange, {getAssignedBroker} from "../../actors/getEnokiExchange";
import {enoki_liquidity_pool_worker} from "../../../../declarations/enoki_liquidity_pool_worker";
import {bigIntToFloat} from "../../utils/utils";
import useLogo from "../../hooks/useLogo";
import useTokenBalance from "../../hooks/useTokenBalance";
import SwitchCheckbox from "../shared/SwitchCheckbox";
import LoadingText from "../shared/LoadingText";
import {setTradeOccurred} from "../../state/lastTradeSlice";
import {Actor} from "@dfinity/agent";
import Order from "./Order";

const ORDER_BOOK_LENGTH = 6;

const priceToFloat = (priceInt, numDecimals) => {
  return Number(priceInt) / Math.pow(10, Number(numDecimals));
}

const OrderBook = () => {
  const {isLoggedIn, getIdentity} = useLogin();
  const [bids, setBids] = React.useState([]);
  const [asks, setAsks] = React.useState([]);

  React.useEffect(() => {
    if (!isLoggedIn) return;

    let stop = false;
    const fetch = () => getEnokiExchange(getIdentity()).getBidAskCurve()
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
      .catch(err => console.error("error retrieving orderbook: ", err));

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
  }, [isLoggedIn])

  return (
    <div className="trades_table">
      <ul className="nav nav-tabs" id="myTab" role="tablist">
        {/*<li className="nav-item" role="presentation">*/}
        {/*  <button className="nav-link active" id="trades_table_tab" data-bs-toggle="tab"*/}
        {/*          data-bs-target="#trades_table"*/}
        {/*          type="button" role="tab" aria-controls="trades_table" aria-selected="true">Trades*/}
        {/*  </button>*/}
        {/*</li>*/}
        <li className="nav-item" role="presentation">
          <button className="nav-link" id="orderbook-tab" data-bs-toggle="tab" data-bs-target="#orderbook_table"
                  type="button" role="tab" aria-controls="orderbook_table" aria-selected="false">Orderbook
          </button>
        </li>
      </ul>
      <div className="tab-content" id="myTabContent">
        <div className="tab-pane show active" id="trades_table" role="tabpanel"
             aria-labelledby="trades_table_tab">
          <div className="trades_table_body">
            <table>
              <tbody>
              <tr>
                <th>Price (eXTC)</th>
                <th>Size (eICP)</th>
              </tr>
              {
                asks.map(([price, amount]) => (
                  <tr key={price.toString()}>
                    <td className="red">{price}</td>
                    <td>{amount.toFixed(0)}</td>
                  </tr>
                ))
              }
              <tr>
                <td></td>
                <td></td>
              </tr>
              {
                bids.map(([price, amount]) => (
                  <tr key={price.toString()}>
                    <td className="green">{price}</td>
                    <td>{amount.toFixed(0)}</td>
                  </tr>
                ))
              }
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  )
}

export default OrderBook;
