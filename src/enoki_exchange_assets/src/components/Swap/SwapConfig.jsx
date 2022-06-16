import React from "react";
import {useSelector, useDispatch} from 'react-redux'
import {setManual, setAuto, setManualValue} from "../../state/swapSlice";
import Tooltip from "../shared/Tooltip";
import SwitchCheckbox from "../shared/SwitchCheckbox";

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

  let inputClass = "";
  if (isError) {
    inputClass += " error_border";
  }
  if (auto) {
    inputClass += " color-gray"
  }

  return (
    <>
      <div className="text-end position-relative">
        <a className="setting" onClick={() => {
          toggleShow()
        }}><img style={{width: 24}} src="img/settings.svg" alt=""/></a>
      </div>
      <div className={"slippage_popup" + (show ? " current" : "")}>
        <div className="overly" onClick={() => toggleShow()}></div>
        <div className={"slippage_body" + (show ? " current" : "")}>
          <h4>Slippage <br/> Tolerance
            <Tooltip style={{display: "inline"}}
                     text={"if your swap cannot be executed at a price within this percentage, it will be cancelled."}>
              <img className="info-icon" style={{marginLeft: 7}} src="img/i.svg" alt=""/>
            </Tooltip>
          </h4>
          <SwitchCheckbox style={{}} checked={!auto} handleOnChange={handleCheckboxChange} textOff="Auto" textOn="Manual"
                          styleOff={{width: 34}} styleOn={{width: 53}}/>
          <div className="input_wrap">
            <input className={inputClass} type="number" min={MIN_SLIPPAGE} max={MAX_SLIPPAGE}
                   value={auto ? currentValue : manualValueStr}
                   onChange={handleChange} disabled={auto}/>
            <div className="symbol">%</div>
          </div>
        </div>
      </div>
    </>
  )
}

export default SwapConfig;
