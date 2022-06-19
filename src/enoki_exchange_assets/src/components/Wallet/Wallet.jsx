import React from "react";
import useLogin from "../../hooks/useLogin";

let shortenPrincipal = principal => {
  if (principal.length > 13) {
    principal = principal.slice(0, 6) + '...' + principal.slice(-5);
  }
  return principal;
}

const Wallet = ({showWallet, toggleShowWallet, showWalletButtons, setShowWalletButtons}) => {
  let {isLoggedIn, getIdentity, login, logout} = useLogin();
  let principal = isLoggedIn ? shortenPrincipal(getIdentity().getPrincipal().toString()) : null;
  const doLogin = provider => {
    login(provider);
    setShowWalletButtons(false);
  }

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
          <a className={`connect wallet-connect${showWalletButtons ? " active" : ""}`} onClick={() => setShowWalletButtons(true)}>CONNECT WALLET</a>
        )
      }
      {showWalletButtons && (
        <>
          <div className="overly right_links_overly" onClick={() => setShowWalletButtons(false)}></div>
          <div className="right_links">
            <a onClick={() => doLogin("nfid")}><img src="img/nfid_icon.png" alt=""/></a>
            {/*<a><img src="img/plug_icon.png" alt=""/></a>*/}
            <a onClick={() => doLogin()}><img src="img/internet-computer-logo.svg" alt=""/></a>
          </div>
        </>
      )}
    </div>
  );
}

export default Wallet;
