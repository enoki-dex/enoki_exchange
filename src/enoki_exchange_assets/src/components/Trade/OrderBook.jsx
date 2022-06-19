import React from "react";

const OrderBook = ({lastPrice, bids, asks}) => {

  let mappingFun;
  if (bids.length || asks.length) {
    let max = bids.concat(asks).reduce((max, next) => Math.max(max, next[1]), 0);
    let min = bids.concat(asks).reduce((min, next) => Math.min(min, next[1]), Number.MAX_VALUE);
    mappingFun = val => {
      if (max - min > 0.01) {
        return 5 + 95 * (val - min) / (max - min);
      } else {
        return 50;
      }
    }
  }

  return (
    <div className="trades_table">
      <h3 style={{
        marginLeft: "17px",
        fontWeight: 600,
        marginBottom: "15px",
        marginTop: "5px"
      }}>Order Book</h3>
      <div className="tab-content" id="myTabContent">
        <div className="tab-pane show active" id="trades_table" role="tabpanel"
             aria-labelledby="trades_table_tab">
          <div className="trades_table_body">
            <table>
              <tbody>
              <tr>
                <th>Price (eXTC)</th>
                <th>Size (eICP)</th>
              </tr>
              {
                asks.map(([price, amount]) => (
                  <tr key={price.toString()}>
                    <td className="red"
                        style={{background: `linear-gradient(to left, var(--red-light-color) 0%, var(--red-light-color) ${((2 * Math.max(0, mappingFun(amount) - 50))).toFixed(0)}%, var(--transparent-color) ${(2 * Math.max(0, mappingFun(amount) - 50)).toFixed(0)}%, var(--transparent-color) 100%)`}}
                    >{price}</td>
                    <td
                      style={{background: `linear-gradient(to left, var(--red-light-color) 0%, var(--red-light-color) ${(2 * Math.min(50, mappingFun(amount))).toFixed(0)}%, var(--transparent-color) ${(2 * Math.min(50, mappingFun(amount))).toFixed(0)}%, var(--transparent-color) 100%)`}}
                    >{amount.toFixed(0)}({mappingFun(amount).toFixed(0)})
                    </td>
                  </tr>
                ))
              }
              <tr>
                <td colSpan={2} style={{textAlign: "center"}}>
                  {lastPrice && (
                    <span className={lastPrice.price_was_lifted ? "green" : "red"}
                          style={{fontSize: "x-large", fontWeight: 600}}><img
                      style={{width: 12, marginRight: 3}}
                      src={lastPrice.price_was_lifted ? "img/dropdown-green.svg" : "img/dropdown-red.svg"}
                      className={lastPrice.price_was_lifted ? "invert-y" : ""}/> {lastPrice.price.toFixed(2)}</span>
                  )}
                </td>
              </tr>
              {
                bids.map(([price, amount]) => (
                  <tr key={price.toString()}>
                    <td className="green"
                        style={{background: `linear-gradient(to left, var(--green-light-color) 0%, var(--green-light-color) ${Math.max(0, mappingFun(amount) - 50).toFixed(0)}%, var(--transparent-color) ${Math.max(0, mappingFun(amount) - 50).toFixed(0)}%, var(--transparent-color) 100%)`}}
                    >{price}</td>
                    <td
                      style={{background: `linear-gradient(to left, var(--green-light-color) 0%, var(--green-light-color) ${(2 * Math.min(50, mappingFun(amount))).toFixed(0)}%, var(--transparent-color) ${(2 * Math.min(50, mappingFun(amount))).toFixed(0)}%, var(--transparent-color) 100%)`}}
                    >{amount.toFixed(0)}</td>
                  </tr>
                ))
              }
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  )
}

export default OrderBook;
