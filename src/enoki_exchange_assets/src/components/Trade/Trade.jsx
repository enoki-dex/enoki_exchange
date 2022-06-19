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
import Orders from "./Orders";
import OrderBook from "./OrderBook";
import useHeartbeat from "../../hooks/useHeartbeat";
import PriceHistory from "./PriceHistory";

const NUM_DECIMALS_QUANTITY = {
  'eICP': 4,
  'eXTC': 2,
}

const MAX_NUMBER_OF_PRICE_DECIMALS = 2; // this is a limitation set by the exchange to reduce the size of the state

const executeOrder = async (identity, canisterId, sellingTokenA, quantity, price, allowTaker) => {
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
    "limitOrder",
    JSON.stringify({allow_taker: allowTaker, limit_price_in_b: price})
  );
  console.log("trade id: ", message);
}

const Trade = ({setShowWalletButtons}) => {
  const dispatch = useDispatch();
  const allowTaker = useSelector(state => state.trade.allowTaker);
  const {isLoggedIn, getIdentity} = useLogin();
  // noinspection JSUnusedLocalSymbols
  const lastExchangeUpdate = useHeartbeat();
  const logoA = useLogo({canisterId: canisterIdA});
  const logoB = useLogo({canisterId: canisterIdB});
  const [side, setSide] = React.useState('buy');
  const [price, setPrice] = React.useState('');
  const [leftQuantity, setLeftQuantity] = React.useState('');
  const [rightQuantity, setRightQuantity] = React.useState('');
  const [lastUpdated, setLastUpdated] = React.useState('left');
  const [usingMax, setUsingMax] = React.useState(null);
  const [isError, setIsError] = React.useState(null);
  const [errorDetails, setErrorDetails] = React.useState(undefined);
  const [extraRewards, setExtraRewards] = React.useState([null, null]);
  const [executing, setExecuting] = React.useState(false);
  const [lastPrices, setLastPrices] = React.useState([]);
  const lastPrice = (lastPrices && lastPrices[lastPrices.length - 1]) || null;

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

  React.useEffect(() => {
    // update data
    let valueStr = lastUpdated === 'right' ? rightQuantity : leftQuantity;
    if (!valueStr) {
      return;
    }
    let value = parseFloat(valueStr);
    if (typeof value !== 'number' || isNaN(value) || value < 0) {
      setIsError(lastUpdated);
      return;
    }
    try {
      let priceVal = parseFloat(price);
      let valLeft = parseFloat(leftQuantity);
      let valRight = parseFloat(rightQuantity);
      if (priceVal && priceVal > 0) {
        if (lastUpdated === 'right') {
          valLeft = valRight / priceVal;
          setLeftQuantity(valLeft.toFixed(4));
        } else {
          valRight = valLeft * priceVal;
          setRightQuantity(valRight.toFixed(4));
        }
      }
      setIsError(false);
      setErrorDetails(undefined);
      if (!usingMax) {
        if (side === 'buy') {
          if (valRight > parseFloat(balancesStr['eXTC'])) {
            setIsError('right');
            setErrorDetails("Insufficient balance");
          }
        } else {
          if (valLeft > parseFloat(balancesStr['eICP'])) {
            setIsError('left');
            setErrorDetails("Insufficient balance");
          }
        }
      }
    } catch (err) {
      console.error(err);
      setIsError(lastUpdated);
      setErrorDetails(err.message);
    }

  }, [lastUpdated, leftQuantity, rightQuantity, price, side]);

  React.useEffect(() => {
    if (!isLoggedIn) return;

    let stop = false;

    getAssignedBroker(getIdentity())
      .then(broker => broker.getAccruedExtraRewards(getIdentity().getPrincipal()))
      .then(rewards => {
        if (stop) return;
        setExtraRewards([bigIntToStr(rewards.token_a, 'eICP', 2), bigIntToStr(rewards.token_b, 'eXTC', 2)])
      })
      .catch(err => console.error("error retrieving extra rewards: ", err));

    return () => {
      stop = true;
    }
  }, [isLoggedIn]);

  React.useEffect(() => {
    let stop = false;
    const fetch = () => getEnokiExchange(undefined).getPriceHistory()
      .then(prices => {
        if (stop) return;
        setLastPrices(prices);
      })
      .catch(err => console.error("error retrieving last prices: ", err));

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
  }, [])

  const handleLeftChange = e => {
    setLeftQuantity(e.target.value);
    setLastUpdated('left');
    setUsingMax(false);
  };
  const handleRightChange = e => {
    setRightQuantity(e.target.value);
    setLastUpdated('right');
    setUsingMax(false);
  };
  const handlePriceChange = e => {
    let newVal;
    if (!e.target.value) {
      newVal = e.target.value;
    } else {
      let val = parseFloat(e.target.value);
      if (!val) return;
      if (Math.floor(val * Math.pow(10, MAX_NUMBER_OF_PRICE_DECIMALS)) === val * Math.pow(10, MAX_NUMBER_OF_PRICE_DECIMALS)) {
        newVal = e.target.value;
      } else {
        newVal = val.toFixed(MAX_NUMBER_OF_PRICE_DECIMALS);
      }
    }
    setPrice(newVal);
    setLastUpdated('price');
    setUsingMax(false);
  };
  const handleSetMax = () => {
    if (side === 'buy') {
      setRightQuantity(balancesStr['eXTC']);
      setLastUpdated('right');
    } else {
      setLeftQuantity(balancesStr['eICP']);
      setLastUpdated('left');
    }
    setUsingMax(true);
  }
  const changeSide = newSide => {
    setSide(newSide);
  }
  const handleToggleAllowTaker = e => {
    if (e.target.checked) {
      dispatch(setAllowTaker())
    } else {
      dispatch(setOnlyMaker())
    }
  }

  const execute = () => {
    let quantity;
    if (usingMax) {
      if (side === 'buy') {
        quantity = balances['eXTC'];
      } else {
        quantity = balances['eICP'];
      }
    } else {
      if (side === 'buy') {
        quantity = floatToBigInt(parseFloat(rightQuantity), 'eXTC');
      } else {
        quantity = floatToBigInt(parseFloat(leftQuantity), 'eICP');
      }
    }
    let limit_price = parseFloat(price);
    let canisterId;
    if (side === 'buy') {
      canisterId = canisterIdB;
    } else {
      canisterId = canisterIdA;
    }

    setExecuting(true);
    executeOrder(getIdentity(), canisterId, side === 'sell', quantity, limit_price, allowTaker)
      .then(() => {
        setLeftQuantity('');
        setRightQuantity('');
      })
      .catch(err => {
        console.error(err);
        setErrorDetails("trade error");
      })
      .then(() => {
        setExecuting(false);
      })
  }

  const readyToTrade = !isError && parseFloat(price) && parseFloat(price) > 0 && parseFloat(leftQuantity) && parseFloat(leftQuantity) > 0;

  return (
    <div className="container">
      <div className="trade_content">
        <div className="content_wrap1">
          <div className="cal_area">
            <div className="eicp_box" style={{cursor: "default"}}>
              <div style={{marginRight: 10}}>
                <img style={{width: 30}} src={logoA} alt=""/>
                <img style={{width: 30}} src={logoB} alt=""/>
              </div>
              <span>eICP/eXTC</span>
            </div>
            <div className="price_line">
              <strong>Current Balance:</strong>{balancesStr['eICP']} eICP / {balancesStr['eXTC']} eXTC<br/><br/>
              <strong>Accrued Market Maker Rewards:</strong>{extraRewards[0] || '--'} eICP
              / {extraRewards[1] || '--'} eXTC
            </div>
            <div className="cal">
              <ul className="nav nav-tabs" id="myTab" role="tablist">
                <li className="nav-item" role="presentation">
                  <button className={`nav-link buy${side === 'buy' ? ' active' : ''}`} id="buy-tab" data-bs-toggle="tab"
                          data-bs-target="#buy" type="button"
                          role="tab"
                          onClick={() => changeSide('buy')}
                          aria-controls="buy" aria-selected={side === 'buy' ? 'true' : 'false'}>BUY
                  </button>
                </li>
                <li className="nav-item" role="presentation">
                  <button className={`nav-link sell${side === 'sell' ? ' active' : ''}`} id="sell-tab"
                          data-bs-toggle="tab" data-bs-target="#sell"
                          type="button"
                          onClick={() => changeSide('sell')}
                          role="tab" aria-controls="sell" aria-selected={side === 'sell' ? 'true' : 'false'}>SELL
                  </button>
                </li>
              </ul>
              <div className="tab-content" id="myTabContent">
                <div className="tab-pane show active" id="sell" role="tabpanel" aria-labelledby="sell-tab">
                  <form action="">
                    <div className="form_group">
                      <label htmlFor="">Amount <a onClick={() => handleSetMax()}>MAX</a> </label>
                      <div className={"input_wrap" + (isError === "left" ? " error_border" : "")}>
                        <input value={leftQuantity} onChange={handleLeftChange} type="number" name="" id="" placeholder="0.0" />
                        <div className="icon"><img src={logoA} alt=""/><span>eICP</span></div>
                      </div>
                    </div>
                    <div className="form_group">
                      <label htmlFor="">Limit Price</label>
                      <div className="input_wrap">
                        <input value={price} onChange={handlePriceChange} type="number" name="" id="" placeholder="0.0" />
                        <div className="icon"><img src={logoB} alt=""/><span>eXTC</span></div>
                      </div>
                    </div>
                    <div className="symbol">=</div>
                    <div className="form_group mt-0">
                      <label htmlFor="">Total</label>
                      <div className={"input_wrap" + (isError === "right" ? " error_border" : "")}>
                        <input value={rightQuantity} onChange={handleRightChange} type="number" name="" id="" placeholder="0.0" />
                        <div className="icon"><img src={logoB} alt=""/><span>eXTC</span></div>
                      </div>
                    </div>
                    <div className="text-end">
                      <SwitchCheckbox style={{
                        justifyContent: "space-around", maxWidth: 250,
                        marginLeft: "auto", marginRight: "auto"
                      }} checked={allowTaker} handleOnChange={handleToggleAllowTaker} textOff="Only Maker"
                                      textOn="Allow Taker"
                                      styleOff={{width: 34}} styleOn={{width: 53}}/>
                    </div>
                    {/*<div className="text-end">*/}
                    {/*<a className="advanced" href="#">Advanced</a>*/}
                    {/*</div>*/}
                    {
                      errorDetails && (
                        <div className="text-center">
                          <span className="error_text">
                            {errorDetails}
                          </span>
                        </div>
                      )
                    }
                    <div className="text-center">
                      {
                        !isLoggedIn ? (
                          <a className="btn btn-black" onClick={() => setShowWalletButtons(true)}>CONNECT WALLET</a>
                        ) : (
                          executing ? (
                            <div style={{position: "absolute", left: "33%", bottom: "5px"}}>
                              <img style={{width: 30, margin: 12}} src="img/spinner.svg"/>
                              <LoadingText style={{fontSize: "large"}} text="Submitting" speed={200}/>
                            </div>
                          ) : (
                            readyToTrade ? (
                              <button className="btn btn-black" onClick={() => execute()}>SUBMIT ORDER</button>
                            ) : (
                              <button className="btn btn-black-disabled">SUBMIT ORDER</button>
                            )
                          )
                        )
                      }
                    </div>
                  </form>
                </div>
              </div>
            </div>
          </div>
          <OrderBook lastPrice={lastPrice}/>
          <PriceHistory lastPrices={lastPrices}/>
        </div>
        <div className="content_wrap2" style={{height: 250}}>
          <Orders/>
        </div>
      </div>
    </div>
  )
}

export default Trade;
