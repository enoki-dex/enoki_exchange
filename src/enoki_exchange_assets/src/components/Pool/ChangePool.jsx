import React from "react";
import Tooltip from "../shared/Tooltip";
import LoadingText from "../shared/LoadingText";
import useLogin from "../../hooks/useLogin";
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import {useDispatch, useSelector} from "react-redux";
import getPoolWorker from "../../actors/getPoolWorker";
import {bigIntToStr, floatToBigInt} from "../../utils/utils";
import {setTradeOccurred} from "../../state/lastTradeSlice";
import {getAssignedTokenShard} from "../../actors/getMainToken";
import {Actor} from "@dfinity/agent";
import useLogo from "../../hooks/useLogo";

const getTokenBalances = async identity => {
    let principal = identity.getPrincipal();
    let [balance_a, balance_b] = await Promise.all([
        getAssignedTokenShard(identity, canisterIdA).then(shard => shard.shardBalanceOf(principal)),
        getAssignedTokenShard(identity, canisterIdB).then(shard => shard.shardBalanceOf(principal))
    ]);
    return [balance_a, balance_b];
}

const getPooledTokenBalances = async identity => {
    let principal = identity.getPrincipal();
    let liquidity = await getPoolWorker(identity).getLiquidity(principal);
    return [liquidity.token_a, liquidity.token_b];
}

const supply = async (identity, quantityA, quantityB) => {
    let worker = getPoolWorker(identity);
    if (!(await worker.isUserRegistered(identity.getPrincipal()))) {
        await worker.register(identity.getPrincipal());
    }
    await Promise.all([
        supplyToken(identity, true, quantityA),
        supplyToken(identity, false, quantityB)
    ]);
    await worker.triggerHeartbeat();
}

const supplyToken = async (identity, isTokenA, quantity) => {
    if (quantity === 0n) {
        return;
    }
    let shard = await getAssignedTokenShard(identity, isTokenA ? canisterIdA : canisterIdB);
    let worker = getPoolWorker(identity);
    let depositShard = isTokenA ? await worker.getAssignedShardA() : await worker.getAssignedShardB();
    let message = await shard.shardTransferAndCall(
        depositShard,
        Actor.canisterIdOf(worker),
        quantity,
        Actor.canisterIdOf(worker),
        "addLiquidity",
        ""
    );
    console.log("liquidity added: ", message);
}

const withdraw = async (identity, quantityA, quantityB, withdrawAll) => {
    let worker = getPoolWorker(identity);
    if (withdrawAll) {
        await worker.removeAllLiquidity();
    } else {
        await worker.removeLiquidity(
            {'token_a': quantityA, 'token_b': quantityB}
        )
    }
    await worker.triggerHeartbeat();
}

const ChangePool = ({adding, setDone}) => {
    const {
        isLoggedIn, getIdentity
    } = useLogin();
    const logoA = useLogo({canisterId: canisterIdA});
    const logoB = useLogo({canisterId: canisterIdB});
    const [balances, setBalances] = React.useState([0, 0]);
    const [balancesStr, setBalancesStr] = React.useState([null, null]);
    const [isErrorA, setIsErrorA] = React.useState(null);
    const [isErrorB, setIsErrorB] = React.useState(null);
    const [errorDetails, setErrorDetails] = React.useState(undefined);
    const lastTradeTime = useSelector(state => state.lastTrade.lastTradeTime);
    const [executing, setExecuting] = React.useState(false);
    const [leftValue, setLeftValue] = React.useState("");
    const [rightValue, setRightValue] = React.useState("");
    const [isMaxA, setIsMaxA] = React.useState(false);
    const [isMaxB, setIsMaxB] = React.useState(false);
    const dispatch = useDispatch();

    let nameBtn, getBalances, nameWaiting, action;
    let nameA = "eICP";
    let nameB = "eXTC";
    if (adding) {
        nameBtn = 'SUPPLY';
        nameWaiting = "Supplying";
        action = supply;
        getBalances = getTokenBalances;
    } else {
        nameBtn = 'REMOVE';
        nameWaiting = "Removing";
        action = withdraw;
        getBalances = getPooledTokenBalances;
        nameA = `pooled ${nameA}`;
        nameB = `pooled ${nameB}`;
    }

    React.useEffect(() => {
        if (!isLoggedIn) return;

        let stop = false;
        getBalances(getIdentity())
            .then(balances => {
                if (stop) return;

                let balanceA = bigIntToStr(balances[0], 'eICP', 2);
                let balanceB = bigIntToStr(balances[1], 'eXTC', 2);
                setBalances(balances);
                setBalancesStr([balanceA, balanceB]);
            })
            .catch(e => console.error("error retrieving change pool balances: ", e));

        return () => {
            stop = true;
        }
    }, [isLoggedIn, lastTradeTime]);

    const handleLeftChange = e => {
        setLeftValue(e.target.value);
        setIsMaxA(false);
        setIsErrorA(false);
        setErrorDetails(undefined);
        let value = parseFloat(e.target.value || '0');
        if (typeof value !== 'number' || isNaN(value)) {
            setIsErrorA(true);
        }
        if (balancesStr[0]) {
            if (parseFloat(balancesStr[0]) < value) {
                setIsErrorA(true);
                setErrorDetails('Insufficient balance');
            }
        }
    };
    const handleRightChange = e => {
        setRightValue(e.target.value);
        setIsMaxB(false);
        setIsErrorB(false);
        setErrorDetails(undefined);
        let value = parseFloat(e.target.value || '0');
        if (typeof value !== 'number' || isNaN(value)) {
            setIsErrorB(true);
        }
        if (balancesStr[1]) {
            if (parseFloat(balancesStr[1]) < value) {
                setIsErrorB(true);
                setErrorDetails('Insufficient balance');
            }
        }
    };

    const execute = () => {
        let quantityA = isMaxA ? balances[0] : floatToBigInt(parseFloat(leftValue), 'eICP');
        let quantityB = isMaxB ? balances[1] : floatToBigInt(parseFloat(rightValue), 'eXTC');
        console.log(adding ? "adding" : "removing", quantityA, quantityB);
        setExecuting(true);
        action(getIdentity(), quantityA, quantityB, isMaxA && isMaxB)
            .then(() => {
                setDone();
            })
            .catch(e => {
                console.error("error executing change pool: ", e);
                setErrorDetails("error with pool worker");
            })
            .then(() => {
                setExecuting(false);
                dispatch(setTradeOccurred());
            })
    }

    const setMaxA = () => {
        setIsErrorA(false);
        setErrorDetails(undefined);
        setLeftValue(balancesStr[0]);
        setIsMaxA(true);
    }
    const setMaxB = () => {
        setIsErrorB(false);
        setErrorDetails(undefined);
        setRightValue(balancesStr[1]);
        setIsMaxB(true);
    }

    const readyToExecute = !isErrorA && !isErrorB && isLoggedIn;

    return (
        <>
            <div className="match_box">
                <div className={"select_wrap" + (isErrorA ? " error_border" : "")}>
                    <div className="input_wrap">
                        <img src={logoA} alt=""/>
                        <h3>eICP</h3>
                    </div>
                    <input type='number' value={leftValue} onChange={handleLeftChange} placeholder="0.0" />
                </div>
                <div className="box_footer">
                    <p>Balance: {balancesStr[0] || "--"} {nameA} <a style={{cursor: "pointer"}}
                                                                    onClick={() => setMaxA()}>MAX</a></p>
                    {/*<p>~$149.71</p>*/}
                </div>
            </div>
            <div className="match_box">
                <a className="top_icon_before" style={{cursor: "default"}}>
                    <img src="img/plus.svg" alt=""/>
                </a>
                <div className={"select_wrap" + (isErrorB ? " error_border" : "")}>
                    <div className="input_wrap">
                        <img src={logoB} alt=""/>
                        <h3>eXTC</h3>
                    </div>
                    <input type='number' value={rightValue} onChange={handleRightChange} placeholder="0.0" />
                </div>
                <div className="box_footer">
                    <p>Balance: {balancesStr[1] || "--"} {nameB} <a style={{cursor: "pointer"}}
                                                                    onClick={() => setMaxB()}>MAX</a></p>
                    {/*<p>~$147.72</p>*/}
                </div>
            </div>
            <div className="cal_details">
                <p>
                    {
                        errorDetails && (
                            <span className="error_text">
                    {errorDetails}
                    </span>
                        )
                    }
                </p>
                <p>You'll earn 0.02%
                    <Tooltip style={{display: "inline"}}
                             text={"You will earn this percentage of all swaps traded."}>
                        <img className="info-icon" style={{marginLeft: 5, marginTop: -3}} src="img/i.svg" alt=""/>
                    </Tooltip>
                </p>
            </div>
            <div className="text-center">
                {
                    isLoggedIn ? (
                        executing ? (
                            <div style={{position: "absolute", left: "45%"}}>
                                <img style={{width: 30, margin: 12}} src="img/spinner.svg"/>
                                <LoadingText style={{fontSize: "large"}} text={nameWaiting} speed={200}/>
                            </div>
                        ) : (
                            readyToExecute ? (
                                <a className="btn connect btn-black btn-big" onClick={() => execute()}>{nameBtn}</a>
                            ) : (
                                <a className="btn connect btn-black-disabled btn-big">{nameBtn}</a>
                            )
                        )
                    ) : (
                        <a className="btn connect btn-black-disabled btn-big">CONNECT WALLET</a>
                    )
                }
            </div>
        </>
    );
}

export default ChangePool;
