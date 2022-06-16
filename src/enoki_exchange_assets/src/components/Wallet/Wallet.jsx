import React from "react";
import useLogin from "../../hooks/useLogin";

let shortenPrincipal = principal => {
  if (principal.length > 13) {
    principal = principal.slice(0, 6) + '...' + principal.slice(-5);
  }
  return principal;
}

const Wallet = ({showWallet, toggleShowWallet}) => {
  let {isLoggedIn, getIdentity, login, logout} = useLogin();
  let principal = isLoggedIn ? shortenPrincipal(getIdentity().getPrincipal().toString()) : null;
  const [walletShow, setWalletShow] = React.useState(false)

  return (
    <div className="col-md-3 col-9 text-end order-2 order-md-3">
      {
        isLoggedIn ? (
          <a style={{cursor: "pointer"}} className="wallet" onClick={() => toggleShowWallet()}>
            <img className="connect-icon" src="img/internet-computer-logo.svg" alt=""/>
            <span>{principal}</span>
            <img className={`arrow${showWallet ? " active" : ""}`} src="img/dropdown.svg" alt=""/>
          </a>
        ) : (
          <a className="connect" onClick={() => login()}>CONNECT WALLET</a>
        )
      }
    </div>
  );
}

export default Wallet;
