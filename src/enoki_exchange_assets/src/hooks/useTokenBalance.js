import React from "react";
import {useSelector} from "react-redux";
import useLogin from "./useLogin";
import {createActor} from "../../../declarations/enoki_wrapped_token";

/**
 *
 * @return BigInt | null
 */
const useTokenBalance = ({principal}) => {
  const {
    isLoggedIn, getIdentity
  } = useLogin();
  const lastTradeTime = useSelector(state => state.lastTrade.lastTradeTime);
  const [balance, setBalance] = React.useState(null);

  React.useEffect(() => {
    if (!isLoggedIn) {
      setBalance(null);
      return;
    }
    let stop = false;
    let tokenActor = createActor(principal);
    tokenActor.balanceOf(getIdentity().getPrincipal())
      .then(balance => {
        if (stop) return;
        setBalance(balance);
      })

    return () => {
      stop = true;
    }
  }, [principal, isLoggedIn, lastTradeTime])


  return balance;
}

export default useTokenBalance;
