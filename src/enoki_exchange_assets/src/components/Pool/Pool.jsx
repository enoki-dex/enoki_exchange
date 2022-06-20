import React from "react";
import useLogin from "../../hooks/useLogin";
import {useSelector} from 'react-redux'
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import {enoki_liquidity_pool_worker} from "../../../../declarations/enoki_liquidity_pool_worker";
import {bigIntToFloat} from "../../utils/utils";
import ChangePool from "./ChangePool";
import useLogo from "../../hooks/useLogo";
import useHeartbeat from "../../hooks/useHeartbeat";

const Pool = ({setShowWalletButtons}) => {
  const {isLoggedIn, getIdentity} = useLogin();
  // noinspection JSUnusedLocalSymbols
  const lastExchangeUpdate = useHeartbeat();
  const logoA = useLogo({canisterId: canisterIdA});
  const logoB = useLogo({canisterId: canisterIdB});
  const lastTradeTime = useSelector(state => state.lastTrade.lastTradeTime);
  const [liquidity, setLiquidity] = React.useState([0, 0]);
  const [netWithdrawals, setNetWithdrawals] = React.useState([0, 0]);
  const [show, setShow] = React.useState(null);
  const [isLoadingBalances, setIsLoadingBalances] = React.useState(false);
  const [isLoadingNetWithdrawals, setIsLoadingNetWithdrawals] = React.useState(false);
  const [waitingForBalanceChange, setWaitingForBalanceChange] = React.useState(false);

  React.useEffect(() => {
    if (!isLoggedIn) {
      setIsLoadingBalances(false);
      setIsLoadingNetWithdrawals(false);
      return;
    }
    let stop = false;
    setIsLoadingBalances(true);
    setIsLoadingNetWithdrawals(true);

    const triggerHeartbeat = async () => {
      await enoki_liquidity_pool_worker.triggerHeartbeat();
    }

    const updateLiquidity = async () => {
      let liquidity = await enoki_liquidity_pool_worker.getLiquidity(getIdentity().getPrincipal());
      if (stop) return false;
      setLiquidity([
        bigIntToFloat(liquidity.token_a, 'eICP'),
        bigIntToFloat(liquidity.token_b, 'eXTC'),
      ]);
      return true;
    }

    const updateNetDeposits = async () => {
      let deposits = await enoki_liquidity_pool_worker.getNetDeposits(getIdentity().getPrincipal());
      if (stop) return false;
      let net_a = bigIntToFloat(deposits.decreased.token_a, 'eICP') - bigIntToFloat(deposits.increased.token_a, 'eICP');
      let net_b = bigIntToFloat(deposits.decreased.token_b, 'eXTC') - bigIntToFloat(deposits.increased.token_b, 'eXTC');
      setNetWithdrawals([net_a, net_b]);
      return true;
    }

    const wait = time => new Promise(resolve => setTimeout(resolve, time));
    const updateLiquidityWhileItDoesntChange = async () => {

      let liquidityOriginal = await enoki_liquidity_pool_worker.getLiquidity(getIdentity().getPrincipal());

      while (!stop) {
        await wait(200);
        await enoki_liquidity_pool_worker.triggerHeartbeat();
        let liquidity = await enoki_liquidity_pool_worker.getLiquidity(getIdentity().getPrincipal());
        if (stop) return false;
        if (liquidity.token_a !== liquidityOriginal.token_a || liquidity.token_b !== liquidityOriginal.token_b) {
          console.log("liquidity changed! from ", liquidityOriginal, "to ", liquidity)
          setLiquidity([
            bigIntToFloat(liquidity.token_a, 'eICP'),
            bigIntToFloat(liquidity.token_b, 'eXTC'),
          ]);
          return true;
        } else {
          console.log("liquidity unchanged from ", liquidityOriginal, "to ", liquidity)
        }
      }
      return false;
    }

    if (waitingForBalanceChange) {
      triggerHeartbeat().then(() => Promise.all([updateLiquidityWhileItDoesntChange(), updateNetDeposits()]))
        .then(([updatedBalances, updatedDeposits]) => {
          if (updatedBalances && updatedDeposits) {
            setIsLoadingBalances(false);
            setIsLoadingNetWithdrawals(false);
            setWaitingForBalanceChange(false);
          }
        })
        .catch(err => console.error("error updating pool values: ", err));
    } else {
      triggerHeartbeat().then(() => Promise.all([updateLiquidity(), updateNetDeposits()]))
        .then(([updatedBalances, updatedDeposits]) => {
          if (updatedBalances) {
            setIsLoadingBalances(false);
          }
          if (updatedDeposits) {
            setIsLoadingNetWithdrawals(false);
          }
        })
        .catch(err => console.error("error updating pool values: ", err));
    }

    return () => {
      stop = true;
    }
  }, [isLoggedIn, lastTradeTime, waitingForBalanceChange]);

  const setActionDone = balancesWillChange => {
    setShow(null);
    if (balancesWillChange) {
      setWaitingForBalanceChange(true);
    }
  }

  let title;
  if (show === 'add') {
    title = "ADD LIQUIDITY";
  } else if (show === 'remove') {
    title = "REMOVE LIQUIDITY";
  } else {
    title = "LIQUIDITY POOL";
  }

  return (
    <div className="container">
      <div className="pool_content">
        <div className="content_wrap">
          <div className="">
            <h1 style={{marginBottom: 27}}>{title}</h1>
            {
              show === 'add' ? (
                <ChangePool adding={true} setDone={setActionDone}/>
              ) : (
                show === 'remove' ? (
                  <ChangePool adding={false} setDone={setActionDone}/>
                ) : (
                  <div className="accordion-item">
                    <div style={{
                      display: "flex",
                      alignItems: "center",
                      padding: "20px 0"
                    }} className="left_icon">
                      <div style={{marginRight: 10}}>
                        <img style={{width: 30}} src={logoA} alt=""/>
                        <img style={{width: 30}} src={logoB} alt=""/>
                      </div>
                      <span className="name">eICP/eXTC</span>
                    </div>
                    <div id="collapse2" className="accordion-collapse collapse show" aria-labelledby="item2"
                         data-bs-parent="#accordion">
                      <div className="accordion-body">
                        <ul>
                          <li>
                            <span>Pooled eICP:</span>
                            <span>{
                              isLoadingBalances ? <img style={{width: 20}} src="img/spinner.svg"/> : liquidity[0]
                            }</span>
                          </li>
                          <li>
                            <span>Pooled eXTC:</span>
                            <span>{
                              isLoadingBalances ? <img style={{width: 20}} src="img/spinner.svg"/> : liquidity[1]
                            }</span>
                          </li>
                          <li>
                            <span>Your net withdrawals:</span>
                            <span>{
                              isLoadingNetWithdrawals ?
                                <img style={{width: 20}} src="img/spinner.svg"/> : netWithdrawals[0].toFixed(2)
                            } eICP / {
                              isLoadingNetWithdrawals ?
                                <img style={{width: 20}} src="img/spinner.svg"/> : netWithdrawals[1].toFixed(2)
                            } eXTC</span>
                          </li>
                        </ul>
                        <div className="btns">
                          {
                            isLoggedIn ? (
                              <>
                                <a className="btn btn-black" onClick={() => setShow('add')}>+ ADD</a>
                                <a className="btn btn-black" onClick={() => setShow('remove')}>- REMOVE</a>
                              </>
                            ) : (
                              <a style={{
                                width: "auto",
                                padding: "12px 25px",
                                marginTop: "20px"
                              }} className="btn connect btn-black btn-big" onClick={() => setShowWalletButtons(true)}>CONNECT
                                WALLET</a>
                            )
                          }
                        </div>
                      </div>
                    </div>
                  </div>
                )
              )
            }

          </div>
        </div>
      </div>
    </div>
  )
}

export default Pool;
