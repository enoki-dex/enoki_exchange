import React from "react";
import {
  HashRouter as Router,
  Redirect,
  Route,
  Switch
} from "react-router-dom";
import store from './store';
import {Provider} from 'react-redux';
import Swap from "./components/Swap/Swap";
import Pool from "./components/Pool/Pool";
import Trade from "./components/Trade/Trade";
import Nav from "./components/Nav/Nav";

import {enoki_exchange} from "../../declarations/enoki_exchange";

const App = () => {
  const [showWalletButtons, setShowWalletButtons] = React.useState(false);
  return (
    <Provider store={store}>
      <Router>
        <Nav showWalletButtons={showWalletButtons} setShowWalletButtons={setShowWalletButtons} />
        <div className="tab-content" id="page_main_tab">
          <Switch>
            <Route path="/swap" render={props => <Swap {...props} showWalletButtons={showWalletButtons} setShowWalletButtons={setShowWalletButtons} />}  />
            <Route path="/pool" render={props => <Pool {...props} showWalletButtons={showWalletButtons} setShowWalletButtons={setShowWalletButtons} />}  />
            <Route path="/trade" render={props => <Trade {...props} showWalletButtons={showWalletButtons} setShowWalletButtons={setShowWalletButtons} />}  />
            {/*<Redirect path="/" exact to="/swap" />*/}
            {/*<Route render={() => <h1>404</h1>} />*/}
            <Redirect to="/swap"/>
          </Switch>
        </div>
      </Router>
    </Provider>
  )
}

export default App;
