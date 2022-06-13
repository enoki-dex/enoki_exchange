import React from "react";
import SwapConfig from "./SwapConfig";
import {useSelector, useDispatch} from 'react-redux'
import {setManual, setAuto, setManualValue} from "../../state/swapSlice";
import Tooltip from "../shared/Tooltip";

const IMAGES = {
  "eICP": "i4.png",
  "eXTC": "i5.png",
};

const Swap = () => {
  const [pair, setPair] = React.useState(['eICP', 'eXTC']);
  const balances = useSelector(state => state.balances);

  const [swapValue, setSwapValue] = React.useState(0.0);
  const [isError, setIsError] = React.useState(false);

  const handleChange = e => {
    let value = parseFloat(e.target.value);
    setIsError(typeof value !== 'number' || isNaN(value) || value < 0);
    setSwapValue(e.target.value);
  };

  const swap = () => {
    setPair([pair[1], pair[0]]);
  }

  return (
    <div className="container">
      <div className="swap_content">
        <div className="content_wrap">
          <h1>SWAP</h1>
          <SwapConfig/>
          <div className="match_box">
            <div className={"select_wrap" + (isError ? " error_border" : "")}>
              <div className="input_wrap">
                {/*<select name="" id="">*/}
                {/*  <option value="">eICP</option>*/}
                {/*  <option value="">eICP</option>*/}
                {/*  <option value="">eICP</option>*/}
                {/*</select>*/}
                <img src={`img/${IMAGES[pair[0]]}`} alt=""/>
                <h3>{pair[0]}</h3>
              </div>
              <input type='number' value={swapValue} onChange={handleChange}/>
            </div>
            <div className="box_footer">
              <p>Balance: {balances[pair[0]]} {pair[0]} <a href="#">DEPOSIT</a></p>
              {/*<p>~$149.71</p>*/}
            </div>
          </div>
          <div className="match_box">
            <a className="top_icon_before" onClick={() => swap()}></a>
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
              <h3>111.199899</h3>
            </div>
            <div className="box_footer">
              <p>Balance: {balances[pair[1]]} {pair[1]}</p>
              {/*<p>~$147.72</p>*/}
            </div>
          </div>
          <div className="cal_details">
            <p>1 eICP = 5.2125464 eXTC</p>
            <p>Fee: 0.03%
              <Tooltip style={{display: "inline"}}
                       text={"Enoki pays for all of your gas, but this small fee is given to Liquidity Providers and Market Makers."}>
                <img style={{marginLeft: 5}} src="img/i6.png" alt=""/>
              </Tooltip>
            </p>
          </div>
          <div className="text-center">
            <a className="btn connect" href="#">CONNECT WALLET</a>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Swap;
