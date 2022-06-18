import {configureStore, combineReducers} from '@reduxjs/toolkit'
import swapReducer from "./state/swapSlice";
import internetIdentityReducer from "./state/internetIdentitySlice";
import lastTradeReducer from "./state/lastTradeSlice";
import tradeReducer from "./state/tradeSlice";

const DEFAULT_II = {
  isLoggedIn: false
};

// convert object to string and store in localStorage
function saveToLocalStorage(state) {
  try {
    state = Object.assign({}, state);
    delete state.ii;
    const serialisedState = JSON.stringify(state);
    localStorage.setItem("enokiPersistentState", serialisedState);
  } catch (e) {
    console.warn(e);
  }
}

// load string from localStarage and convert into an Object
// invalid output must be undefined
function loadFromLocalStorage() {
  try {
    const serialisedState = localStorage.getItem("enokiPersistentState");
    if (serialisedState === null) return undefined;
    const state = JSON.parse(serialisedState);
    state.ii = DEFAULT_II;
    return state;
  } catch (e) {
    console.warn(e);
    return undefined;
  }
}

const appReducer = combineReducers({
  swap: swapReducer,
  ii: internetIdentityReducer,
  lastTrade: lastTradeReducer,
  trade: tradeReducer
});

const rootReducer = (state, action) => {
  if (action.type === 'USER_LOGOUT') {
    localStorage.clear();
    return appReducer(undefined, action);
  }

  return appReducer(state, action)
};

const store = configureStore({
  reducer: rootReducer,
  preloadedState: loadFromLocalStorage()
});

store.subscribe(() => {
  saveToLocalStorage(store.getState());
})

export default store;
