import React from "react";
import {useSelector} from "react-redux";
import useLogin from "./useLogin";
import {getAssignedTokenShard} from "../actors/getMainToken";

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
    getAssignedTokenShard(getIdentity(), principal)
        .then(shard => shard.shardBalanceOf(getIdentity().getPrincipal()))
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
