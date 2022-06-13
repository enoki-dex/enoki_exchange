import React from "react";
import {useSelector, useDispatch} from 'react-redux'
import {setManual, setAuto, setManualValue} from "../../state/swapSlice";
import Tooltip from "../shared/Tooltip";

const MIN_SLIPPAGE = 0.01;
const MAX_SLIPPAGE = 30;

const SwapConfig = () => {
  const config = useSelector(state => state.swap);
  const dispatch = useDispatch();
  const [show, setShow] = React.useState(false);

  const {auto, currentValue} = config.slippage;
  const [manualValueStr, setManualValueStr] = React.useState(config.slippage.manualValue);
  const [isError, setIsError] = React.useState(false);

  const parseNumber = strVal => {
    let value = parseFloat(strVal);
    if (typeof value === 'number' && !isNaN(value) && value <= MAX_SLIPPAGE && value >= MIN_SLIPPAGE) {
      return value;
    } else {
      return NaN;
    }
  }

  const handleChange = e => {
    let value = parseNumber(e.target.value);
    setIsError(isNaN(value));
    setManualValueStr(e.target.value);
  };

  const handleCheckboxChange = e => {
    if (e.target.checked) {
      dispatch(setManual());
    } else {
      setIsError(false);
      setAuto();
      dispatch(setAuto());
      let value = parseNumber(manualValueStr);
      if (isNaN(value)) {
        setManualValueStr(config.slippage.manualValue);
      }
    }
  }

  const autoSwitchRef = React.useRef();

  const toggleShow = () => {
    if (show) {
      // save now
      if (!auto) {
        let value = parseNumber(manualValueStr);
        if (!isNaN(value)) {
          dispatch(setManualValue(value));
        }
      }
    }

    setShow(!show);
  }


  return (
    <>
      <div className="text-end position-relative">
        <a className="setting" onClick={() => {
          toggleShow()
        }}><img src="img/i1.png" alt=""/></a>
      </div>
      <div className={"slippage_popup" + (show ? " current" : "")}>
        <div className="overly" onClick={() => toggleShow()}></div>
        <div className={"slippage_body" + (show ? " current" : "")}>
          <h4>Slippage <br/> Tolerance
            <Tooltip style={{display: "inline"}}
                     text={"if your swap cannot be executed at a price within this percentage, it will be cancelled."}>
              <img style={{marginLeft: 7}} src="img/i6.png" alt=""/>
            </Tooltip>
          </h4>
          <div className="mode-switch">
            <p>Auto</p>
            <div>
              <input type="checkbox" id="switch" ref={autoSwitchRef}
                     checked={!auto}
                     onChange={handleCheckboxChange}/>
              <label htmlFor="switch"></label>
            </div>
            <p className="active">Manual</p>
          </div>
          <div className="input_wrap">
            <input className={isError ? "error_border" : ""} type="number" min={MIN_SLIPPAGE} max={MAX_SLIPPAGE} value={auto ? currentValue : manualValueStr}
                   onChange={handleChange} disabled={auto}/>
            <div className="symbol">%</div>
          </div>
        </div>
      </div>
    </>
  )
}

export default SwapConfig;
