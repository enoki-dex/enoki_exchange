import React from "react";

const style =
  {
    display: "flex",
    background: "#8b8b8beb",
    fontWeight: "bold",
    fontSize: "large",
    color: "white",
    userSelect: "none",
    top: 0,
    bottom: 0,
    left: 0,
    right: 0,
    position: "absolute",
    justifyContent: "space-around",
    alignItems: "center",
    zIndex: 1000,
    borderRadius: 5,
  };

const ComingSoon = ({text, customStyle}) => {
  const newStyle = Object.assign({}, style, customStyle);
  if (typeof text === 'undefined') {
    text = "COMING SOON!";
  }

  return (
    <div className="coming_soon" style={newStyle}>
      {text}
    </div>
  );
}

export default ComingSoon;
