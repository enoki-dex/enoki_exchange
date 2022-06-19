import React from "react";
import {useSelector} from "react-redux";
import useLogin from "./useLogin";
import {getAssignedTokenShardPrincipal} from "../actors/getMainToken";
import getTokenShard from "../actors/getTokenShard";

/**
 *
 * @return BigInt | null
 */
const useTokenBalance = ({principal}) => {
  const {
    isLoggedIn, getIdentity
  } = useLogin();
  const lastTradeTime = useSelector(state => state.lastTrade.lastTradeTime);
  const [assignedShard, setAssignedShard] = React.useState(null);
  const [balance, setBalance] = React.useState(null);

  React.useEffect(() => {
    if (!isLoggedIn) {
      setBalance(null);
      return;
    }

    let stop = false;
    getAssignedTokenShardPrincipal(getIdentity(), principal)
      .then(shard => {
        if (stop) return;
        setAssignedShard(shard);
      })
      .catch(e => console.error(`error getting assigned shard for ${principal}: `, e));

    return () => {
      stop = true;
    }
  }, [principal]);

  React.useEffect(() => {
    if (!isLoggedIn || !assignedShard) {
      setBalance(null);
      return;
    }

    let stop = false;
    const wait = time => new Promise(resolve => setTimeout(resolve, time));
    const run = async () => {
      while (!stop) {
        let balance = await getTokenShard(getIdentity(), assignedShard).shardBalanceOf(getIdentity().getPrincipal());
        if (stop) return;
        setBalance(balance);
        await wait(10000);
      }
    }

    run()
      .catch(err => console.error(`error updating token balance for ${principal}:`, err));

    return () => {
      stop = true;
    }
  }, [assignedShard, principal, isLoggedIn, lastTradeTime])


  return balance;
}

export default useTokenBalance;
