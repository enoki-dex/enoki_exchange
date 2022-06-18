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

const PriceHistory = ({lastPrices}) => {


  return (
    <div className="chart">
      <div className="select">
        <p>eICP/eXTC</p>
        {/*<select name="" id="">*/}
        {/*  <option value="">1D</option>*/}
        {/*  <option value="">2D</option>*/}
        {/*  <option value="">3D</option>*/}
        {/*</select>*/}
      </div>
      <img src="img/chart.png" width="100%" alt=""/>
    </div>
  )
}

export default PriceHistory;
