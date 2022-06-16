import React from "react";

const SwitchCheckbox = ({textOff, textOn, style = {}, styleOff = {}, styleOn = {}, checked, handleOnChange}) => {

  return (
    <div style={style} className="mode-switch">
      <p style={styleOff} className={!checked ? "active" : ""}>{textOff}</p>
      <div>
        <label className={checked ? "input-checked" : ""}>
          <input type="checkbox"
                 checked={checked}
                 onChange={handleOnChange}/>
        </label>
      </div>
      <p style={styleOn} className={checked ? "active" : ""}>{textOn}</p>
    </div>
  );
}

export default SwitchCheckbox;
