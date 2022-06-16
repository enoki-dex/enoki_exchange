import React from "react";
import SwapConfig from "./SwapConfig";
import Tooltip from "../shared/Tooltip";
import {useDispatch, useSelector} from 'react-redux'
import {enoki_exchange} from "../../../../declarations/enoki_exchange";
import getEnokiExchange from "../../actors/getEnokiExchange";
import useLogin from "../../hooks/useLogin";
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import useTokenBalance from "../../hooks/useTokenBalance";
import {bigIntToStr, floatToBigInt} from "../../utils/utils";
import LoadingText from "../shared/LoadingText";
import {getAssignedTokenShard} from "../../actors/getMainToken";

const IMAGES = {
  "eICP": "icp_test.svg",
  "eXTC": "xtc_test.svg",
};

const NUM_DECIMALS_QUANTITY = {
  'eICP': 4,
  'eXTC': 2,
}

//TODO: get from backend
const prices = {
  "eICP;eXTC": {
    price: 6.00,
    decimalsPrice: 2,
    decimalsQuantity: NUM_DECIMALS_QUANTITY['eICP'],
  },
  "eXTC;eICP": {
    price: 0.167,
    decimalsPrice: 4,
    decimalsQuantity: NUM_DECIMALS_QUANTITY['eXTC'],
  },
}

const getPriceAndQuantity = ({
                               buy, sell, buyQuantity, sellQuantity
                             }) => {
  let key = [buy, sell].join(';');
  let inverseKey = [sell, buy].join(';');
  if (!prices[key] || !prices[inverseKey]) {
    throw new Error("cannot find price");
  }
  let decimalsPrice;
  let decimalsQuantity;
  let price;
  if (typeof buyQuantity === 'undefined') {
    ({decimalsPrice, decimalsQuantity, price} = prices[key]);
    buyQuantity = sellQuantity / price;
  } else {
    ({price, decimalsPrice} = prices[key]);
    sellQuantity = buyQuantity * price;
    ({decimalsQuantity} = prices[inverseKey]);
  }
  return {
    price,
    buyQuantity,
    sellQuantity,
    decimalsPrice,
    decimalsQuantity,
  }
}

const execute_swap = async (identity, canisterId, quantity, price) => {
  let shard = await getAssignedTokenShard(identity, canisterId);
  // shard.shardTransferAndCall()
  // dfx --identity swapper1 canister call "$assigned_shard_b_3" shardTransferAndCall "(principal \"$deposit_shard_b_3\", principal \"$broker_3\", 50_000_000 : nat, principal \"$broker_3\", \"swap\", \"{\\\"allow_taker\\\": true, \\\"limit_price_in_b\\\": 6.1}\")"
}

const Swap = () => {
  const [pair, setPair] = React.useState(['eICP', 'eXTC']);

  const [leftSwapValue, setLeftSwapValue] = React.useState("0.0");
  const [rightSwapValue, setRightSwapValue] = React.useState("0.0");
  const [price, setPrice] = React.useState(undefined);
  const [priceDecimals, setPriceDecimals] = React.useState(undefined);
  const [isError, setIsError] = React.useState(null);
  const [errorDetails, setErrorDetails] = React.useState(undefined);
  const [usingMax, setUsingMax] = React.useState(false);
  const slippage = useSelector(state => state.swap.slippage.currentValue);
  const [executingSwap, setExecutingSwap] = React.useState(false);

  const balances = {
    'eICP': useTokenBalance({principal: canisterIdA}),
    'eXTC': useTokenBalance({principal: canisterIdB})
  };

  const balancesStr = {};
  Object.keys(balances).forEach(token => {
    let balance = balances[token];
    if (balance !== null) {
      balancesStr[token] = bigIntToStr(balance, token, NUM_DECIMALS_QUANTITY[token]);
    }
  });

  let {isLoggedIn, getIdentity} = useLogin();

  const setMax = () => {
    let value = balancesStr[pair[0]] || '0';
    setLeftSwapValue(value)
    updateData(true, value, pair);
    setUsingMax(true);
  }

  const updateData = (editingLeft, newValue, newPair) => {
    setErrorDetails(undefined);
    let value = parseFloat(newValue);
    let valueLeft = value;
    if (typeof value !== 'number' || isNaN(value) || value < 0) {
      setIsError(editingLeft ? "left" : "right");
      setPrice(undefined);
    } else {
      setIsError(null);
      let input = editingLeft ? {sellQuantity: value} : {buyQuantity: value};

      try {
        let {
          price,
          buyQuantity,
          sellQuantity,
          decimalsPrice,
          decimalsQuantity,
        } = getPriceAndQuantity(Object.assign(input, {sell: newPair[0], buy: newPair[1]}));
        if (editingLeft) {
          setRightSwapValue(buyQuantity.toFixed(decimalsQuantity));
        } else {
          valueLeft = sellQuantity;
          setLeftSwapValue(sellQuantity.toFixed(decimalsQuantity));
        }
        setPrice(price);
        setPriceDecimals(decimalsPrice);
        if (valueLeft > parseFloat(balancesStr[newPair[0]])) {
          setIsError('left');
          setErrorDetails("Insufficient balance");
        }
      } catch (err) {
        console.error(err);
        setIsError(editingLeft ? "left" : "right");
        setErrorDetails(err.message);
      }
    }
  }

  const handleLeftChange = e => {
    setLeftSwapValue(e.target.value);
    updateData(true, e.target.value, pair);
    setUsingMax(false);
  };
  const handleRightChange = e => {
    setRightSwapValue(e.target.value);
    updateData(false, e.target.value, pair);
    setUsingMax(false);
  };

  const switchPair = () => {
    setPair([pair[1], pair[0]]);
    let [left, right] = [rightSwapValue, leftSwapValue];
    setLeftSwapValue(left);
    setRightSwapValue(right);
    updateData(true, left, [pair[1], pair[0]]);
    setUsingMax(false);
  }

  const swap = () => {
    let quantity = usingMax ?
      balances[pair[0]] :
      floatToBigInt(parseFloat(leftSwapValue), pair[0]);
    let limit_price = price * (1 + (slippage / 100));
    let canisterId;
    if (pair[0] === 'eICP') {
      canisterId = canisterIdA;
      limit_price = 1 / limit_price;
    } else {
      canisterId = canisterIdB;
    }
    setExecutingSwap(true);
    execute_swap(getIdentity(), canisterId, quantity, limit_price)
      .catch(err => {
        console.error(err);
      })
      .then(() => setExecutingSwap(false));
  }

  return (
    <div className="container">
      <div className="swap_content">
        <div className="content_wrap">
          <h1>SWAP</h1>
          <SwapConfig/>
          <div className="match_box">
            <div className={"select_wrap" + (isError === "left" ? " error_border" : "")}>
              <div className="input_wrap">
                {/*<select name="" id="">*/}
                {/*  <option value="">eICP</option>*/}
                {/*  <option value="">eICP</option>*/}
                {/*  <option value="">eICP</option>*/}
                {/*</select>*/}
                <img src={`img/${IMAGES[pair[0]]}`} alt=""/>
                <h3>{pair[0]}</h3>
              </div>
              <input type='number' value={leftSwapValue} onChange={handleLeftChange}/>
            </div>
            <div className="box_footer">
              <p>Balance: {balancesStr[pair[0]] || "--"} {pair[0]} <a style={{cursor: "pointer"}}
                                                                      onClick={() => setMax()}>MAX</a></p>
              {/*<p>~$149.71</p>*/}
            </div>
          </div>
          <div className="match_box">
            <a className="top_icon_before" onClick={() => switchPair()}>
              <img src="img/swap_icon.svg" alt=""/>
            </a>
            <div className="select_wrap">
              <div className="input_wrap">
                {/*<select name="" id="">*/}
                {/*  <option value="">eICP</option>*/}
                {/*  <option value="">eICP</option>*/}
                {/*  <option value="">eICP</option>*/}
                {/*</select>*/}
                <img src={`img/${IMAGES[pair[1]]}`} alt=""/>
                <h3>{pair[1]}</h3>
              </div>
              <input type='number' value={rightSwapValue} onChange={handleRightChange}/>
            </div>
            <div className="box_footer">
              <p>Balance: {balancesStr[pair[1]] || "--"} {pair[1]}</p>
              {/*<p>~$147.72</p>*/}
            </div>
          </div>
          <div className="cal_details">
            <p>{typeof price !== 'undefined' && (
              <span>1 {pair[1]} = {price.toFixed(priceDecimals)} {pair[0]}</span>)}</p>
            <p>Fee: 0.03%
              <Tooltip style={{display: "inline"}}
                       text={"Enoki pays for all of your gas, but this small fee is given to Liquidity Providers and Market Makers."}>
                <img className="info-icon" style={{marginLeft: 5, marginTop: -3}} src="img/i.svg" alt=""/>
              </Tooltip>
            </p>
          </div>
          <div className="text-center">
            {
              isLoggedIn ? (
                executingSwap ? (
                  <div style={{position: "absolute", left: "45%"}}>
                    <img style={{width: 30, margin: 12}} src="img/spinner.svg"/>
                    <LoadingText style={{fontSize: "large"}} text="Swapping" speed={200}/>
                  </div>
                ) : (
                  <a className="btn connect btn-black btn-big" onClick={() => swap()}>SWAP</a>
                )
              ) : (
                <a className="btn connect btn-black-disabled btn-big">CONNECT WALLET</a>
              )
            }
          </div>
        </div>
      </div>
    </div>
  )
}

export default Swap;
