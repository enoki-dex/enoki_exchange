import React from "react";
import SwapConfig from "./SwapConfig";
import {useSelector, useDispatch} from 'react-redux'
import Tooltip from "../shared/Tooltip";
import {enoki_exchange} from "../../../../declarations/enoki_exchange";
import getEnokiExchange from "../../actors/getEnokiExchange";
import useLogin from "../../hooks/useLogin";
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import useTokenBalance from "../../hooks/useTokenBalance";
import {bigIntToStr} from "../../utils/utils";

const IMAGES = {
  "eICP": "i4.png",
  "eXTC": "i5.png",
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

const Swap = () => {
  const [pair, setPair] = React.useState(['eICP', 'eXTC']);

  const [leftSwapValue, setLeftSwapValue] = React.useState("0.0");
  const [rightSwapValue, setRightSwapValue] = React.useState("0.0");
  const [price, setPrice] = React.useState(undefined);
  const [priceDecimals, setPriceDecimals] = React.useState(undefined);
  const [isError, setIsError] = React.useState(null);
  const [errorDetails, setErrorDetails] = React.useState(undefined);

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

  let {isLoggedIn, getIdentity, login, logout} = useLogin();

  const updateData = (editingLeft, newValue, newPair) => {
    setErrorDetails(undefined);
    let value = parseFloat(newValue);
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
          setLeftSwapValue(sellQuantity.toFixed(decimalsQuantity));
        }
        setPrice(price);
        setPriceDecimals(decimalsPrice);
      } catch (err) {
        console.error(err);
        setErrorDetails(err.message);
      }
    }
  }

  const handleLeftChange = e => {
    setLeftSwapValue(e.target.value);
    updateData(true, e.target.value, pair);
  };
  const handleRightChange = e => {
    setRightSwapValue(e.target.value);
    updateData(false, e.target.value, pair);
  };

  const switchPair = () => {
    setPair([pair[1], pair[0]]);
    let [left, right] = [rightSwapValue, leftSwapValue];
    setLeftSwapValue(left);
    setRightSwapValue(right);
    updateData(true, left, [pair[1], pair[0]]);
  }

  const swap = () => {
    if (isLoggedIn) {
      let client = getEnokiExchange(getIdentity());
      client.whoami().then(me => console.log("ME LOGGED IN: " + me.toString()))
      client.whoisanon().then(me => console.log("ANON: " + me.toString()))
    } else {
      enoki_exchange.whoami().then(me => console.log("ME: " + me.toString()))
      enoki_exchange.whoisanon().then(me => console.log("ANON: " + me.toString()))
    }
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
              <p>Balance: {balancesStr[pair[0]] || "--"} {pair[0]} <a
                href="#">DEPOSIT</a></p>
              {/*<p>~$149.71</p>*/}
            </div>
          </div>
          <div className="match_box">
            <a className="top_icon_before" onClick={() => switchPair()}></a>
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
                <img style={{marginLeft: 5}} src="img/i6.png" alt=""/>
              </Tooltip>
            </p>
          </div>
          <div className="text-center">
            {
              isLoggedIn ? (
                <a className="btn connect btn-black" onClick={() => swap()}>SWAP</a>
              ) : (
                <a className="btn connect btn-black" onClick={() => login()}>CONNECT WALLET</a>
              )
            }
          </div>
        </div>
      </div>
    </div>
  )
}

export default Swap;
