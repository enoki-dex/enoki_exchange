import React from "react";
import {useDispatch} from "react-redux";
import useLogin from "../../hooks/useLogin";
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import useTokenBalance from "../../hooks/useTokenBalance";
import {bigIntToStr} from "../../utils/utils";
import ComingSoon from "../shared/ComingSoon";
import {getAssignedTokenShard} from "../../actors/getMainToken";
import getTokenShard from "../../actors/getTokenShard";
import {setTradeOccurred} from "../../state/lastTradeSlice";
import LoadingText from "../shared/LoadingText";
import useLogo from "../../hooks/useLogo";

const WalletModal = ({toggleShowWallet}) => {
  let {logout, getIdentity} = useLogin();
  const logoA = useLogo({canisterId: canisterIdA});
  const logoB = useLogo({canisterId: canisterIdB});
  const dispatch = useDispatch();
  const balanceEIcp = useTokenBalance({principal: canisterIdA});
  const balanceEXtc = useTokenBalance({principal: canisterIdB});
  const balanceEIcpStr = balanceEIcp !== null && bigIntToStr(balanceEIcp, 'eICP', 6, null);
  const balanceEXtcStr = balanceEXtc !== null && bigIntToStr(balanceEXtc, 'eXTC', 4, null);

  const [mintingA, setMintingA] = React.useState(false);
  const [mintingB, setMintingB] = React.useState(false);

  const mintA = () => {
    setMintingA(true);
    mint(canisterIdA, () => setMintingA(false));
  }

  const mintB = () => {
    setMintingB(true);
    mint(canisterIdB, () => setMintingB(false));
  }

  const mint = (principal, cb) => {
    getAssignedTokenShard(getIdentity(), principal)
      .then(assignedShard => assignedShard.mint(BigInt("1000000000000000")))
      .catch(e => console.error(e))
      .then(() => {
        cb();
        dispatch(setTradeOccurred());
      })
  }

  const clickLogout = () => {
    toggleShowWallet();
    logout();
  }

  return (
    <div className="wallet-modal">
      <div className="overlay" onClick={() => toggleShowWallet()}></div>
      <div className="modal-dialog">
        <div className="modal-content">
          <div className="modal-header">
            <h4>Wallet</h4>
            <a style={{cursor: "pointer"}} onClick={() => clickLogout()}>Disconnect Wallet</a>
          </div>
          <div className="modal-body">
            <div className="box">
              <h5>Enoki-Boosted Tokens</h5>
              <div className="icon_box">
                <img className="icon" src={logoA} alt=""/>
                <div className="content">
                  <p><b>{balanceEIcpStr !== null ? balanceEIcpStr : "--"}</b> eICP</p>
                  {
                    mintingA ? (
                      <div>
                        <img style={{width: 20, margin: 8}} src="img/spinner.svg"/>
                        <LoadingText text="Minting" speed={200}/>
                      </div>
                    ) : (
                      <button onClick={() => mintA()} className="btn btn-small btn-black" data-bs-toggle="modal"
                              data-bs-target="#unboost-modal">MINT</button>
                    )
                  }
                </div>
              </div>
              <div className="icon_box">
                <img className="icon" src={logoB} alt=""/>
                <div className="content">
                  <p><b>{balanceEXtcStr !== null ? balanceEXtcStr : "--"}</b> eXTC</p>
                  {
                    mintingB ? (
                      <div>
                        <img style={{width: 20, margin: 8}} src="img/spinner.svg"/>
                        <LoadingText text="Minting" speed={200}/>
                      </div>
                    ) : (
                      <button onClick={() => mintB()} className="btn btn-small btn-black" data-bs-toggle="modal"
                              data-bs-target="#unboost-modal">MINT</button>
                    )
                  }
                </div>
              </div>
            </div>
            <div className="box">
              <h5>Tokens</h5>
              <div className="icon_box">
                <img className="icon" src="img/icp_1.svg" alt=""/>
                <div className="content">
                  <p><b>0.0</b> ICP</p>
                  <button className="btn"><img src="img/i17.png" alt=""/> BOOST</button>
                </div>
              </div>
              <div className="icon_box">
                <img className="icon" src="img/xtc_1.svg" alt=""/>
                <div className="content">
                  <p><b>0.0</b> XTC</p>
                  <button className="btn" data-bs-toggle="modal" data-bs-target="#boost-modal"><img
                    src="img/i17.png" alt=""/> BOOST
                  </button>
                </div>
              </div>
              <ComingSoon customStyle={{width: "50%", left: "50%"}}/>
            </div>
          </div>
          <div className="modal-footer">
            <a className="btn btn-big btn-black-disabled" data-bs-toggle="modal" data-bs-target="#deposit-modal">+
              DEPOSIT</a>
            <a className="btn btn-big btn-black-disabled" data-bs-toggle="modal" data-bs-target="#send-modal">SEND</a>
          </div>
        </div>
      </div>
    </div>
  );
}

export default WalletModal;
