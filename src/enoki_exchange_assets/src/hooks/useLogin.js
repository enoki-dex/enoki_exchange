import React from "react";
import {AuthClient} from "@dfinity/auth-client";
import {useSelector, useDispatch} from "react-redux";
import {setLoggedIn, setLoggedOut} from "../state/internetIdentitySlice";
import {canisterId} from "../../../declarations/internet_identity";

let iiUrl;
if (process.env.NODE_ENV !== "production") {
  iiUrl = `http://localhost:8000/?canisterId=${canisterId}`;
} else {
  iiUrl = `https://identity.ic0.app`;
}

let authClient = null;
let gotStatus = false;

// 7 day in nanoseconds
const MAX_TIME_TO_LIVE = BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000);

const useLogin = () => {
  const {isLoggedIn} = useSelector(state => state.ii);
  const dispatch = useDispatch();

  React.useEffect(() => {
    if (gotStatus) {
      return;
    }
    gotStatus = true;
    AuthClient.create().then(client => {
      authClient = client;
      client.isAuthenticated().then(yes => {
        if (yes) {
          dispatch(setLoggedIn())
        }
      })
    })
  }, []);

  const getIdentity = () => {
    return isLoggedIn ? authClient.getIdentity() : null;
  }

  const login = () => {
    if (!authClient) {
      console.error("auth client still initializing...");
      return;
    }
    const _ = authClient.login({
      identityProvider: iiUrl,
      maxTimeToLive: MAX_TIME_TO_LIVE,
      onSuccess: async () => {
        dispatch(setLoggedIn());
      },
    });
  };
  const logout = () => {
    const _ = authClient.logout();
    localStorage.clear();
    dispatch(setLoggedOut())
  }

  return {
    isLoggedIn, getIdentity, login, logout
  };
}

export default useLogin;
