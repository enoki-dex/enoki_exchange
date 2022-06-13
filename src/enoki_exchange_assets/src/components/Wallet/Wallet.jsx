import React from "react";

const Wallet = () => {
  return (
    <div className="col-md-3 col-9 text-end order-2 order-md-3">
      <a className="connect" href="#">CONNECT WALLET</a>
      <a className="wallet" href="#">
        <img src="img/i8.png" alt=""/>
        <span>J343i...18g82</span>
        <img className="arrow" src="img/bottom-arrow.png" alt=""/>
      </a>
    </div>
  );
}

export default Wallet;
