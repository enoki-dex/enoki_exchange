import React from "react";
import {NavLink} from "react-router-dom";
import Wallet from "../Wallet/Wallet";

const Nav = () => {
  return (
    <header>
      <div className="container position-relative">
        <div className="row align-items-center">
          <div className="col-md-3 col-3 text-start order-1 order-md-1">
            <a className="logo" href="/"><img src="img/logo.svg" className="img-fluid" alt="" style={{ width: 84 }}/></a>
          </div>
          <div className="col-md-6 col-12 text-center order-3 order-md-2">
            <ul className="nav nav-tabs" id="myTab" role="tablist">
              <li className="nav-item" role="presentation">
                <NavLink exact to="/swap" className="nav-link" activeClassName="active">
                  SWAP
                </NavLink>
              </li>
              <li className="nav-item" role="presentation">
                <NavLink exact to="/pool" className="nav-link" activeClassName="active">
                  POOL
                </NavLink>
              </li>
              <li className="nav-item" role="presentation">
                <NavLink exact to="/trade" className="nav-link" activeClassName="active">
                  TRADE
                </NavLink>
              </li>
            </ul>
          </div>
          <Wallet/>
        </div>

      </div>
    </header>
  );
}

export default Nav;
