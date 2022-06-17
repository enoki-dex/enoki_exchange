import React from "react";
import {bigIntToStr} from "../../utils/utils";
import LoadingText from "../shared/LoadingText";

const MAX_NUMBER_OF_PRICE_DECIMALS = 2; // this is a limitation set by the exchange to reduce the size of the state

const Order = ({order, pendingCancel, isPending, cancel}) => {
  const quantityToStr = quantity => {
    return bigIntToStr(quantity, 'eICP', 2);
  }

  const [cancelling, setCancelling] = React.useState(false);
  const [statusIsCancelling, setStatusIsCancelling] = React.useState(false);

  let id, sideObj, amount, price, status;
  if (isPending) {
    id = order.id;
    if (pendingCancel[id]) {
      status = "Cancelling"
    } else {
      status = "Pending";
    }
    sideObj = order.side;
    amount = quantityToStr(order.quantity);
    price = order.limit_price;
  } else {
    id = order.info.id;
    sideObj = order.info.side;
    status = Object.keys(order.state.status)[0];
    amount = `${quantityToStr(order.state.quantity_a_executed)} (${Math.round(order.state.fraction_executed * 100)}%)`
    price = order.state.average_price;
  }

  let side = Object.keys(sideObj)[0].toUpperCase();

  const tryCancel = () => {
    setCancelling(true);
    cancel(id)
      .then(() => {
        setCancelling(false);
        setStatusIsCancelling(true);
      })
      .catch(e => console.error("could not cancel order: ", e));
  }

  if (statusIsCancelling) {
    status = "Cancelling";
  }

  return (
    <tr>
      <td>{status}</td>
      <td>eICP/eXTC</td>
      <td>{side}</td>
      <td>{amount}</td>
      <td>{price}</td>
      {/*<td>2022-06-01 15:32 PM</td>*/}
      <td style={{height: 46}}>
        {isPending && (
          <>
            {cancelling ? (
              <div style={{position: "absolute", left: "45%"}}>
                <img style={{width: 10, margin: 3}} src="img/spinner.svg"/>
                <LoadingText style={{}} text="Cancelling" speed={200}/>
              </div>
            ) : (
              <a className="btn" onClick={() => tryCancel()}>CANCEL</a>
            )}
          </>
        )}
      </td>
    </tr>
  );
}

export default Order;
