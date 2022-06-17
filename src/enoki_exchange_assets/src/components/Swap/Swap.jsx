import React from "react";
import SwapConfig from "./SwapConfig";
import Tooltip from "../shared/Tooltip";
import {useDispatch, useSelector} from 'react-redux'
import {getAssignedBroker} from "../../actors/getEnokiExchange";
import useLogin from "../../hooks/useLogin";
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import useTokenBalance from "../../hooks/useTokenBalance";
import {bigIntToStr, floatToBigInt} from "../../utils/utils";
import LoadingText from "../shared/LoadingText";
import {getAssignedTokenShard} from "../../actors/getMainToken";
import {Actor} from "@dfinity/agent";
import {setTradeOccurred} from "../../state/lastTradeSlice";
import useLogo from "../../hooks/useLogo";
import useHeartbeat from "../../hooks/useHeartbeat";

const NUM_DECIMALS_QUANTITY = {
  'eICP': 4,
  'eXTC': 2,
}

const decimal_formatting = {
  "eICP;eXTC": {
    decimalsPrice: 2,
    decimalsQuantity: NUM_DECIMALS_QUANTITY['eICP'],
  },
  "eXTC;eICP": {
    decimalsPrice: 4,
    decimalsQuantity: NUM_DECIMALS_QUANTITY['eXTC'],
  },
}

const getPriceAndQuantityDecimals = ({buy, sell}) => {
  let key = [buy, sell].join(';');
  let inverseKey = [sell, buy].join(';');
  if (!decimal_formatting[key] || !decimal_formatting[inverseKey]) {
    throw new Error("cannot find decimal_formatting");
  }
  let {decimalsPrice, decimalsQuantity: decimalsQuantityRight} = decimal_formatting[key];
  let {decimalsQuantity: decimalsQuantityLeft} = decimal_formatting[inverseKey];
  return {
    decimalsPrice,
    decimalsQuantityLeft,
    decimalsQuantityRight,
  }
}

const execute_swap = async (identity, canisterId, sellingTokenA, quantity, price) => {
  let shard = await getAssignedTokenShard(identity, canisterId);
  let broker = await getAssignedBroker(identity);
  if (!(await broker.isUserRegistered(identity.getPrincipal()))) {
    await broker.register(identity.getPrincipal());
  }
  let broker_shard = sellingTokenA ? await broker.getAssignedShardA() : await broker.getAssignedShardB();
  let message = await shard.shardTransferAndCall(
    broker_shard,
    Actor.canisterIdOf(broker),
    quantity,
    Actor.canisterIdOf(broker),
    "swap",
    JSON.stringify({allow_taker: true, limit_price_in_b: price})
  );
  console.log("swap success: ", message);
}

const Swap = () => {
  const {isLoggedIn, getIdentity} = useLogin();
  const lastExchangeUpdate = useHeartbeat();
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
  const [lastUpdatedLeft, setLastUpdatedLeft] = React.useState(true);
  const [isFetching, setIsFetching] = React.useState(false);
  const dispatch = useDispatch();
  const logoA = useLogo({canisterId: canisterIdA});
  const logoB = useLogo({canisterId: canisterIdB});

  const logos = {
    'eICP': logoA,
    'eXTC': logoB,
  }

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


  const setMax = () => {
    let value = balancesStr[pair[0]] || '0';
    setLeftSwapValue(value)
    setLastUpdatedLeft(true);
    setUsingMax(true);
  }

  React.useEffect(() => {
    console.log("canisterIdA", canisterIdA);
    console.log("canisterIdB", canisterIdB);
    // update data
    let value = parseFloat(lastUpdatedLeft ? leftSwapValue : rightSwapValue);
    if (typeof value !== 'number' || isNaN(value) || value < 0) {
      setIsError(lastUpdatedLeft ? "left" : "right");
      return;
    }
    try {
      let {
        decimalsPrice,
        decimalsQuantityLeft,
        decimalsQuantityRight,
      } = getPriceAndQuantityDecimals({sell: pair[0], buy: pair[1]});

      let valueLeft = null;
      if (lastUpdatedLeft) {
        valueLeft = value;
        if (typeof price !== 'undefined' && price !== null) {
          let valueRight = pair[1] === 'eICP' ? value / price : value * price;
          setRightSwapValue(valueRight.toFixed(decimalsQuantityRight));
        }
      } else {
        if (typeof price !== 'undefined' && price !== null) {
          valueLeft = pair[1] === 'eICP' ? value * price : value / price;
          setLeftSwapValue(valueLeft.toFixed(decimalsQuantityLeft));
        }
      }
      setPriceDecimals(decimalsPrice);
      if (!usingMax && valueLeft !== null && valueLeft > parseFloat(balancesStr[pair[0]])) {
        setIsError('left');
        setErrorDetails("Insufficient balance");
      } else {
        setIsError(false);
        setErrorDetails(undefined);
      }
    } catch (err) {
      console.error(err);
      setIsError(lastUpdatedLeft ? "left" : "right");
      setErrorDetails(err.message);
    }

  }, [lastUpdatedLeft, leftSwapValue, rightSwapValue, pair, price]);

  React.useEffect(() => {
    if (!isLoggedIn) {
      return;
    }
    let stop = false;
    setIsFetching(true);
    let quantity = floatToBigInt(parseFloat(leftSwapValue) || 0.001, pair[0]);
    getAssignedBroker(getIdentity())
      .then(broker => broker.getExpectedSwapPrice(
        pair[0] === 'eICP' ? {'Sell': null} : {'Buy': null},
        quantity
      ))
      .then(price => {
        if (stop) return;
        setPrice(price);
      })
      .catch(err => {
        console.error(err);
      })
      .then(() => {
        if (stop) return;
        setIsFetching(false);
      });

    return () => {
      stop = true;
    }
  }, [isLoggedIn, leftSwapValue, pair])

  const handleLeftChange = e => {
    setLeftSwapValue(e.target.value);
    setLastUpdatedLeft(true);
    setUsingMax(false);
  };
  const handleRightChange = e => {
    setRightSwapValue(e.target.value);
    setLastUpdatedLeft(false);
    setUsingMax(false);
  };

  const switchPair = () => {
    setPair([pair[1], pair[0]]);
    let [left, right] = [rightSwapValue, leftSwapValue];
    setLeftSwapValue(left);
    setRightSwapValue(right);
    setLastUpdatedLeft(true);
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
    } else {
      canisterId = canisterIdB;
    }
    setExecutingSwap(true);
    execute_swap(getIdentity(), canisterId, pair[0] === 'eICP', quantity, limit_price)
      .then(() => {
        setLeftSwapValue("0");
        setRightSwapValue("0");
      })
      .catch(err => {
        console.error(err);
        setErrorDetails("swap error");
      })
      .then(() => {
        setExecutingSwap(false);
        dispatch(setTradeOccurred());
      })
  }

  const priceInRightToken = pair[1] === 'eICP' ? price : (price && 1 / price);
  const readyToExecuteSwap = isLoggedIn && !isError && !isFetching && price;

  return (
    <div className="container">
      <div className="swap_content">
        <div className="content_wrap">
          <h1>SWAP</h1>
          <SwapConfig/>
          <div className="match_box">
            <div className={"select_wrap" + (isError === "left" ? " error_border" : "")}>
              <div className="input_wrap">
                <img src={logos[pair[0]]} alt=""/>
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
            <a className="top_icon_before clickable" onClick={() => switchPair()}>
              <img src="img/swap_icon.svg" alt=""/>
            </a>
            <div className={"select_wrap" + (isError === "right" ? " error_border" : "")}>
              <div className="input_wrap">
                <img src={logos[pair[1]]} alt=""/>
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
            <p>
              {
                errorDetails ? (
                  <span className="error_text">
                    {errorDetails}
                    </span>
                ) : (
                  <span>
                    1 {pair[1]} = {
                    isFetching ? (
                      <img style={{width: 17, margin: "0 3px 3px"}} src="img/spinner.svg"/>
                    ) : (
                      (typeof price !== 'undefined' && price !== null) ? priceInRightToken.toFixed(priceDecimals) :
                        <span>&nbsp;--</span>
                    )
                  } {pair[0]}
                  </span>
                )
              }
            </p>
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
                  readyToExecuteSwap ? (
                    <a className="btn connect btn-black btn-big" onClick={() => swap()}>SWAP</a>
                  ) : (
                    <a className="btn connect btn-black-disabled btn-big">SWAP</a>
                  )
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
