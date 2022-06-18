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

const APPLICATION_NAME = "Enoki";
const APPLICATION_LOGO_URL = "https://enoki.ooo/favicon.ico"

const logOutAction = () => {
  return {
    type: "USER_LOGOUT",
    payload: {}
  }
}

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
          dispatch(setLoggedIn());
          console.log('logged in:', authClient.getIdentity().getPrincipal().toString());
        }
      })
    })
  }, []);

  const getIdentity = () => {
    return isLoggedIn ? authClient.getIdentity() : null;
  }

  const login = provider => {
    if (!authClient) {
      console.error("auth client still initializing...");
      return;
    }
    dispatch(logOutAction());

    const url = provider === 'nfid' ?
      `https://nfid.one/authenticate/?applicationName=${encodeURIComponent(APPLICATION_NAME)}&applicationLogo=${encodeURIComponent(APPLICATION_LOGO_URL)}#authorize` :
      iiUrl;

    const _ = authClient.login({
      identityProvider: url,
      maxTimeToLive: MAX_TIME_TO_LIVE,
      onSuccess: async () => {
        dispatch(setLoggedIn());
        console.log('logged in:', authClient.getIdentity().getPrincipal().toString());
      },
      windowOpenerFeatures:
        `left=${window.screen.width / 2 - 200}, ` +
        `top=${window.screen.height / 2 - 300},` +
        `toolbar=0,location=0,menubar=0,width=400,height=600`
    });
  };
  const logout = () => {
    authClient.logout()
      .catch(e => console.error("error logging out: ", e))
      .then(() => {
        dispatch(logOutAction());
      });
  }

  return {
    isLoggedIn, getIdentity, login, logout
  };
}

export default useLogin;
