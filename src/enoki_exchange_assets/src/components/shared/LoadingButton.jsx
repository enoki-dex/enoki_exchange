import React from "react";

const defaultStyle =
  {
    position: "relative",
    color: "rgba(0,0,0,0)"
  };

const ComingSoon = ({hiddenText, style, className}) => {
  const newStyle = Object.assign({}, defaultStyle, style);

  return (
    <button style={newStyle} className={`btn btn-black-disabled${className && ` ${className}`}`}>{hiddenText}<img
      style={{width: 18, height: 18, margin: "auto", position: "absolute", left: 0, right: 0, top: 0, bottom: 0}}
      src="img/spinner_white.svg"/></button>
  );
}

export default ComingSoon;
