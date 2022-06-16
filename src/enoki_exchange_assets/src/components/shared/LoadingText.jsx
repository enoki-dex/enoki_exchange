import React from "react";


const LoadingText = ({text, speed = 300, style = {}}) => {
  const [value, setValue] = React.useState(text);
  const defaultStyle = {
    userSelect: "none",
  };
  const appliedStyle = Object.assign(defaultStyle, style);

  React.useEffect(() => {
    const handle = window.setInterval(() => {
      setValue(c => (c === text + "..." ? text : c + "."));
    }, speed);
    return () => {
      window.clearInterval(handle);
    };
  }, [text, speed]);

  return <span style={appliedStyle}>{value}</span>;
}

export default LoadingText;
