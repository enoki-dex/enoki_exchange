import React from "react";
import useLogin from "../../hooks/useLogin";
import {useSelector} from 'react-redux'
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import {bigIntToStr, floatToBigInt} from "../../utils/utils";
import {getAssignedTokenShard} from "../../actors/getMainToken";
import {enoki_liquidity_pool_worker} from "../../../../declarations/enoki_liquidity_pool_worker";
import {bigIntToFloat} from "../../utils/utils";
import ChangePool from "./ChangePool";

const Pool = () => {
  const {isLoggedIn, getIdentity} = useLogin();
  const lastTradeTime = useSelector(state => state.lastTrade.lastTradeTime);
  const [liquidity, setLiquidity] = React.useState([0, 0]);
  const [netWithdrawals, setNetWithdrawals] = React.useState([0, 0]);
  const [show, setShow] = React.useState(null);
  const [isLoadingBalances, setIsLoadingBalances] = React.useState(true);
  const [isLoadingNetWithdrawals, setIsLoadingNetWithdrawals] = React.useState(true);

  React.useEffect(() => {
    if (!isLoggedIn) {
      return;
    }
    let stop = false;

    //TODO: remove if/when we switch to canister hearbeat
    enoki_liquidity_pool_worker.triggerHeartbeat().then(() => {
      enoki_liquidity_pool_worker.getLiquidity(getIdentity().getPrincipal())
        .then(liquidity => {
          if (stop) return;

          setLiquidity([
            bigIntToFloat(liquidity.token_a, 'eICP'),
            bigIntToFloat(liquidity.token_b, 'eXTC'),
          ]);
          setIsLoadingBalances(false);
        })
        .catch(e => console.error("error getting liquidity: ", e));

      enoki_liquidity_pool_worker.getNetDeposits(getIdentity().getPrincipal())
        .then(deposits => {
          if (stop) return;

          let net_a = bigIntToFloat(deposits.decreased.token_a, 'eICP') - bigIntToFloat(deposits.increased.token_a, 'eICP');
          let net_b = bigIntToFloat(deposits.decreased.token_b, 'eXTC') - bigIntToFloat(deposits.increased.token_b, 'eXTC');
          setNetWithdrawals([net_a, net_b]);
          setIsLoadingNetWithdrawals(false);
        })
        .catch(e => console.error("error getting net deposits: ", e));
    })
      .catch(e => console.error("error triggering heartbeat: ", e));

    return () => {
      stop = true;
    }
  }, [isLoggedIn, lastTradeTime]);

  const setActionDone = () => {
    setShow(null);
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
            <h1 style={{marginBottom: 25}}>{title}</h1>
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
                        <img style={{width: 30}} src="img/icp_test.svg" alt=""/>
                        <img style={{width: 30}} src="img/xtc_test.svg" alt=""/>
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
                              isLoadingBalances ? <img style={{width: 20}} src="img/spinner.svg" /> : liquidity[0]
                            }</span>
                          </li>
                          <li>
                            <span>Pooled eXTC:</span>
                            <span>{
                              isLoadingBalances ? <img style={{width: 20}} src="img/spinner.svg" /> : liquidity[1]
                            }</span>
                          </li>
                          <li>
                            <span>Your net withdrawals:</span>
                            <span>{
                              isLoadingNetWithdrawals ? <img style={{width: 20}} src="img/spinner.svg" /> : netWithdrawals[0].toFixed(2)
                            } eICP / {
                              isLoadingNetWithdrawals ? <img style={{width: 20}} src="img/spinner.svg" /> : netWithdrawals[1].toFixed(2)
                            } eXTC</span>
                          </li>
                        </ul>
                        <div className="btns">
                          <a className="btn btn-black" onClick={() => setShow('add')}>+ ADD</a>
                          <a className="btn btn-black" onClick={() => setShow('remove')}>- REMOVE</a>
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
