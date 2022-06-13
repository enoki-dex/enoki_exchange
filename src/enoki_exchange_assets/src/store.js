import { configureStore } from '@reduxjs/toolkit'
import swapReducer from "./state/swapSlice";
import balancesReducer from "./state/balancesSlice";

// convert object to string and store in localStorage
function saveToLocalStorage(state) {
  try {
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
    return JSON.parse(serialisedState);
  } catch (e) {
    console.warn(e);
    return undefined;
  }
}

const store = configureStore({
  reducer: {
    swap: swapReducer,
    balances: balancesReducer,
  },
  preloadedState: loadFromLocalStorage()
});

store.subscribe(() => {
 saveToLocalStorage(store.getState());
})

export default store;
