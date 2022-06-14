import React from "react";


const LoadingText = ({text, speed = 300}) => {
  let [value, setValue] = React.useState(text);

  React.useEffect(() => {
    const handle = window.setInterval(() => {
      setValue(c => (c === text + "..." ? text : c + "."));
    }, speed);
    return () => {
      window.clearInterval(handle);
    };
  }, [text, speed]);

  return <span>{value}</span>;
}

export default LoadingText;
