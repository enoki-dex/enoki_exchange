import React from "react";
import useLogin from "../../hooks/useLogin";
import {useSelector, useDispatch} from 'react-redux'
import {setAllowTaker, setOnlyMaker} from "../../state/tradeSlice";
import {canisterId as canisterIdA} from "../../../../declarations/enoki_wrapped_token";
import {canisterId as canisterIdB} from "../../../../declarations/enoki_wrapped_token_b";
import {bigIntToStr, floatToBigInt} from "../../utils/utils";
import {getAssignedTokenShard} from "../../actors/getMainToken";
import {getAssignedBroker} from "../../actors/getEnokiExchange";
import {enoki_liquidity_pool_worker} from "../../../../declarations/enoki_liquidity_pool_worker";
import {bigIntToFloat} from "../../utils/utils";
import useLogo from "../../hooks/useLogo";
import useTokenBalance from "../../hooks/useTokenBalance";
import SwitchCheckbox from "../shared/SwitchCheckbox";
import LoadingText from "../shared/LoadingText";
import {setTradeOccurred} from "../../state/lastTradeSlice";
import {Actor} from "@dfinity/agent";
import Order from "./Order";

const Orders = () => {
  const dispatch = useDispatch();
  const {isLoggedIn, getIdentity} = useLogin();
  const [orders, setOrders] = React.useState([]);
  const [pendingCancel, setPendingCancel] = React.useState({});
  const [pastOrders, setPastOrders] = React.useState([]);
  const [showingOpen, setShowingOpen] = React.useState(true);

  const cancelOrder = id => {
    return getAssignedBroker(getIdentity())
      .then(broker => broker.cancelOrder(id));
  }

  React.useEffect(() => {
    if (!isLoggedIn) return;

    let stop = false;
    const fetch = () => getAssignedBroker(getIdentity())
      .then(broker => {
        return Promise.all(
          [
            broker.getOpenOrders(getIdentity().getPrincipal()),
            broker.getPastOrders(getIdentity().getPrincipal())
          ]
        )
      })
      .then(([open, past]) => {
        if (stop) return;
        let open_orders = open.open_orders;
        open_orders.sort((a, b) => {
          if (a.id < b.id) {
            return -1;
          } else if (a.id > b.id) {
            return 1;
          } else {
            return 0;
          }
        });
        setOrders(open_orders);
        let pendingCancel = {};
        open.pending_cancel.forEach(id => pendingCancel[id] = true);
        setPendingCancel(pendingCancel);
        past.sort((a, b) => {
          if (a.info.id < b.info.id) {
            return 1;
          } else if (a.info.id > b.info.id) {
            return -1;
          } else {
            return 0;
          }
        });
        setPastOrders(past);
      })
      .catch(err => console.error("error retrieving orders: ", err));
    const wait = delay => new Promise(resolve => setTimeout(resolve, delay));
    const run = async () => {
      while (!stop) {
        await fetch();
        await wait(5000);
      }
    }

    let _ = run();

    return () => {
      stop = true;
    }
  }, [isLoggedIn])

  let list = showingOpen ? orders : pastOrders;

  return (
    <div className="openOrder_table">
      <ul className="nav nav-tabs" id="myTab" role="tablist">
        <li className="nav-item" role="presentation">
          <button onClick={() => setShowingOpen(true)} className={`nav-link${showingOpen ? ' active' : ''}`}
                  id="open-orders-tab" data-bs-toggle="tab"
                  data-bs-target="#open-orders" type="button" role="tab" aria-controls="open-orders"
                  aria-selected={showingOpen ? 'true' : 'false'}>Open
            Orders {(orders.length && `(${orders.length})`) || ''}
          </button>
        </li>
        <li className="nav-item" role="presentation">
          <button onClick={() => setShowingOpen(false)} className={`nav-link${!showingOpen ? ' active' : ''}`}
                  id="past-orders-tab" data-bs-toggle="tab"
                  data-bs-target="#past-orders"
                  type="button" role="tab" aria-controls="past-orders"
                  aria-selected={!showingOpen ? 'true' : 'false'}>Past Orders
          </button>
        </li>
      </ul>
      <div className="tab-content" id="myTabContent">
        <div className="tab-pane show active" id="open-orders" role="tabpanel" aria-labelledby="open-orders-tab">
          <div className="openOrder_table_body">
            <div className="wrapper1">
              <div className="wrapper2">
                <table>
                  <tbody>
                  <tr>
                    <th>Status</th>
                    <th>Product</th>
                    <th>Side</th>
                    <th>Amount</th>
                    <th>Price</th>
                    {/*<th>Good till</th>*/}
                    <th></th>
                  </tr>
                  {
                    list.map(order => (
                      <Order key={order.id || order.info.id} order={order} isPending={showingOpen}
                             pendingCancel={pendingCancel} cancel={id => cancelOrder(id)}/>
                    ))
                  }
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Orders;
