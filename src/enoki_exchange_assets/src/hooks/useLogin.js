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
const APPLICATION_LOGO_URL = "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiIHN0YW5kYWxvbmU9Im5vIj8+CjwhLS0gQ3JlYXRlZCB3aXRoIElua3NjYXBlIChodHRwOi8vd3d3Lmlua3NjYXBlLm9yZy8pIC0tPgoKPHN2ZwogICB2ZXJzaW9uPSIxLjEiCiAgIGlkPSJzdmc2MiIKICAgd2lkdGg9IjUxMiIKICAgaGVpZ2h0PSI1MTIiCiAgIHZpZXdCb3g9IjAgMCA1MTIgNTEyIgogICBzb2RpcG9kaTpkb2NuYW1lPSJmYXZfaWNvbl81MTJfY29sb3JlZC5wbmciCiAgIGlua3NjYXBlOnZlcnNpb249IjEuMiAoZGMyYWVkYSwgMjAyMi0wNS0xNSkiCiAgIHhtbG5zOmlua3NjYXBlPSJodHRwOi8vd3d3Lmlua3NjYXBlLm9yZy9uYW1lc3BhY2VzL2lua3NjYXBlIgogICB4bWxuczpzb2RpcG9kaT0iaHR0cDovL3NvZGlwb2RpLnNvdXJjZWZvcmdlLm5ldC9EVEQvc29kaXBvZGktMC5kdGQiCiAgIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIKICAgeG1sbnM6c3ZnPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CiAgPGRlZnMKICAgICBpZD0iZGVmczY2IiAvPgogIDxzb2RpcG9kaTpuYW1lZHZpZXcKICAgICBpZD0ibmFtZWR2aWV3NjQiCiAgICAgcGFnZWNvbG9yPSIjZmZmZmZmIgogICAgIGJvcmRlcmNvbG9yPSIjMDAwMDAwIgogICAgIGJvcmRlcm9wYWNpdHk9IjAuMjUiCiAgICAgaW5rc2NhcGU6c2hvd3BhZ2VzaGFkb3c9IjIiCiAgICAgaW5rc2NhcGU6cGFnZW9wYWNpdHk9IjAuMCIKICAgICBpbmtzY2FwZTpwYWdlY2hlY2tlcmJvYXJkPSIwIgogICAgIGlua3NjYXBlOmRlc2tjb2xvcj0iI2QxZDFkMSIKICAgICBzaG93Z3JpZD0iZmFsc2UiCiAgICAgaW5rc2NhcGU6em9vbT0iMC44MzE1NjUzMyIKICAgICBpbmtzY2FwZTpjeD0iMjI2LjY4MDkzIgogICAgIGlua3NjYXBlOmN5PSIzMDAuNjM3ODQiCiAgICAgaW5rc2NhcGU6d2luZG93LXdpZHRoPSIxMzkwIgogICAgIGlua3NjYXBlOndpbmRvdy1oZWlnaHQ9Ijg2NiIKICAgICBpbmtzY2FwZTp3aW5kb3cteD0iMCIKICAgICBpbmtzY2FwZTp3aW5kb3cteT0iMzgiCiAgICAgaW5rc2NhcGU6d2luZG93LW1heGltaXplZD0iMCIKICAgICBpbmtzY2FwZTpjdXJyZW50LWxheWVyPSJnNjgiIC8+CiAgPGcKICAgICBpbmtzY2FwZTpncm91cG1vZGU9ImxheWVyIgogICAgIGlua3NjYXBlOmxhYmVsPSJJbWFnZSIKICAgICBpZD0iZzY4Ij4KICAgIDxwYXRoCiAgICAgICBzdHlsZT0iZmlsbDojZDRiMjk5O2ZpbGwtb3BhY2l0eToxO3N0cm9rZTojZDRiMjk5O3N0cm9rZS13aWR0aDoxLjgzNDUxO3N0cm9rZS1vcGFjaXR5OjEiCiAgICAgICBkPSJtIDI0Ny4wODA4NSw0NzcuMDM1NTcgYyAtOC40ODc1OCwtMi43MDIyMyAtMTcuOTQzOTEsLTExLjA3MTI4IC0yMS41OTU1OSwtMTkuMTEyNTIgLTMuMjM4ODUsLTcuMTMyMTkgLTMuNDMxMjksLTE0LjM5MzkxIC0zLjQ2OTM0LC0xMzAuOTIxNzYgbCAtMC4wNDAzLC0xMjMuMzcwNzUgaCAtOC4wMDQ5IGMgLTIwLjAwNTYzLDAgLTQzLjQ3MjUyLC04LjczMTAxIC01MS4zNTk2MiwtMTkuMTA4NjcgLTguMjU4ODksLTEwLjg2Njg3IC0xMC43NzkzMywtMTguMDA1OTEgLTExLjcyODU0LC0zMy4yMjA2MiAtMS4zMTg5OSwtMjEuMTQxODMgMS45NDA1MiwtMzcuMjk2MDQgMTEuNTI1NzQsLTU3LjEyMTgzMyAyMS41NzIzLC00NC42MTk0OTIgNzEuMDI2MDcsLTY5LjYxOTYxNSAxMTUuMjU1MDUsLTU4LjI2NDMyOCAyMy45ODE2OSw2LjE1NzAyNCA0Ni4wNjkyNSwyMC43NzcyMzYgNjAuNzU4MSw0MC4yMTcwNDggMjUuODY4NTEsMzQuMjM1NDAzIDMwLjMxNTE5LDg3Ljc4MDA1MyA5LjEyMjExLDEwOS44NDM1NjMgLTguMTQ0NDEsOC40Nzg5MSAtMjUuMzkyOTMsMTUuNDk5NjIgLTQyLjA5Nzc3LDE3LjEzNTE3IGwgLTE0LjY3NjA3LDEuNDM2OTIgLTAuOTE3MjUsMTI2LjYyNzE0IC0wLjkxNzI2LDEyNi42MjcxMyAtNC4xOTg1OCw1LjMzNjQ4IGMgLTkuNDM5ODQsMTEuOTk4MjYgLTI1LjI2OTkxLDE3Ljg0MDQyIC0zNy42NTU3OSwxMy44OTcwMyB6IgogICAgICAgaWQ9InBhdGgzMTYiIC8+CiAgICA8cGF0aAogICAgICAgc3R5bGU9ImZpbGw6I2Q0YjI5OTtzdHJva2Utd2lkdGg6MS44MzQ1MTtzdHJva2U6I2Q0YjI5OTtzdHJva2Utb3BhY2l0eToxO2ZpbGwtb3BhY2l0eToxIgogICAgICAgZD0ibSAzNDQuODg3NzYsNDc3LjU1NTE4IGMgLTguNjY4OTQsLTMuNjUyOCAtMTYuOTEyNDIsLTEwLjY4OTEyIC0yMC4yOTgxOSwtMTcuMzI1NzkgLTYuMDgzNDMsLTExLjkyNDQ5IC01LjMwMTc3LC0xNi40NTIwMiAxNS40NzkwNywtODkuNjU4NSAzLjU4MDE5LC0xMi42MTIyNiA3LjM2NzQxLC0yNS44MTUyNyA4LjQxNjA0LC0yOS4zNDAwNCAyLjcxNjgzLC05LjEzMjA4IDIuNzgyMTcsLTkuMDA1MDYgLTYuODA3MzIsLTEzLjIzMTY2IC0xMi43NzEyNCwtNS42Mjg5NyAtMjAuNzY5OTgsLTEyLjM3MTMgLTI0LjU1MTIsLTIwLjY5NDc5IC0xMC44MzE3OSwtMjMuODQzNzEgNi4yOTk2MSwtNjMuODcyNjkgMzUuNzk2NjgsLTgzLjY0MjAzIDQ0LjYyMTUzLC0yOS45MDU5NiAxMDEuMTE4NjYsLTExLjMyMzI2IDEyMC42NzcyOCwzOS42OTI0NSA2LjA4NTE5LDE1Ljg3MjMxIDUuOTk5NjgsNDEuMjU0MjEgLTAuMTkzOCw1Ny41MTk0MyAtOC40ODc0NSwyMi4yODk2NiAtMjEuOTY0MTYsMzAuODg1MzggLTQ1LjQ5MzQ4LDI5LjAxNjcxIGwgLTExLjQ1MjE1LC0wLjkwOTUyIC0xLjk0NTM1LDcuNTg0MzMgYyAtMS4wNjk5NCw0LjE3MTM4IC0zLjkyNTQyLDE0LjYwMTMzIC02LjM0NTUxLDIzLjE3NzY2IC01LjA3MjgxLDE3Ljk3NzA2IC04LjMyMjU3LDI5Ljk1NjA3IC0xNS40MjgwNyw1Ni44Njk3OSAtNS44ODIyMSwyMi4yODAyMyAtOS44NjAzNywyOS42MzMxMSAtMTkuNjIwOSwzNi4yNjU1NiAtNy4wMjM5Myw0Ljc3Mjg5IC0yMi4wODQwOSw3LjI2NzM5IC0yOC4yMzMxLDQuNjc2NCB6IgogICAgICAgaWQ9InBhdGgzMTgiIC8+CiAgICA8cGF0aAogICAgICAgc3R5bGU9ImZpbGw6I2Q0YjI5OTtzdHJva2Utd2lkdGg6MS44MzQ1MTtzdHJva2U6I2Q0YjI5OTtzdHJva2Utb3BhY2l0eToxO2ZpbGwtb3BhY2l0eToxIgogICAgICAgZD0ibSAxNDcuNjc4LDQ3Ny4yNDgxOSBjIC05LjkwMzIyLC0zLjIyMDEzIC0xNi4wMDkzMSwtOC45MDM1MyAtMjAuOTkxMjMsLTE5LjUzODExIC0yLjgzNjAzLC02LjA1Mzg4IC01LjcwNDY4LC0xMy40ODM2NCAtNi4zNzQ3OCwtMTYuNTEwNTggLTEuNjI0MDksLTcuMzM2MjMgLTEyLjc5OTA1LC00OC40MTg1OSAtMTkuNzczMjcsLTcyLjY5MjIgbCAtNS42MDAyMTEsLTE5LjQ5MTQzIC0xMS40NDQ4MDUsMC45MTg2OCBjIC0yNC43NTA3MDgsMS45ODY3NSAtMzguMDIxOTAzLC03LjgwNzc5IC00Ni4xMTM5MjgsLTM0LjAzMzQ1IC01LjU1MzE2NSwtMTcuOTk3NCAtNS41MDQ4MDIsLTMwLjM1NjggMC4xOTE4NywtNDkuMDMzMjYgNy4xNjgzNjYsLTIzLjUwMTQgMjAuNDMzNzE4LC0zOC45MjcwNCA0Mi45MjcyMjUsLTQ5LjkxODAyIDI5LjkwODk5OSwtMTQuNjE0NCA2NS40OTA2MTksLTguMTQzNTkgOTAuNTYxMzU5LDE2LjQ2OTMxIDEyLjQ5MTU0LDEyLjI2MzQzIDIwLjY1OTc3LDI2LjYzMDc1IDI1LjE3MzA4LDQ0LjI3NzYgMy4yNjExNSwxMi43NTEgMy4zMzEyNCwxNC42NjEwNiAwLjg0NTEzLDIzLjAzMjcgLTMuNzIwNTgsMTIuNTI4NTYgLTkuODU3NDcsMTkuMTI5NzEgLTIzLjk5MzU2LDI1LjgwODY5IC05LjE3Mzg3LDQuMzM0NDUgLTExLjgxNTA4LDYuNDI1MjMgLTExLjEyNDc4LDguODA2MzggMy4zNjI4LDExLjU5OTY5IDE0LjkzMjIxLDUzLjE4OTYgMTcuOTY0MzQsNjQuNTc4NTQgMi4wMTQ3LDcuNTY3MzUgNS4zNzg1NSwxOS40MTY1MyA3LjQ3NTI0LDI2LjMzMTUzIDQuOTI5ODcsMTYuMjU5MDIgNC4wNjY3LDI5LjY1MDQxIC0yLjQ2Mjk5LDM4LjIxMTI3IC04LjM0OTQ3LDEwLjk0NjcyIC0yNS4xNjM2NiwxNi43MTUxNyAtMzcuMjU4NjksMTIuNzgyMzUgeiIKICAgICAgIGlkPSJwYXRoMzIwIiAvPgogIDwvZz4KPC9zdmc+Cg=="

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

  const login = provider => {
    if (!authClient) {
      console.error("auth client still initializing...");
      return;
    }
    const url = provider === 'nfid' ?
      `https://nfid.one/authenticate/?applicationName=${encodeURIComponent(APPLICATION_NAME)}&applicationLogo=${encodeURIComponent(APPLICATION_LOGO_URL)}#authorize` :
      iiUrl;

    const _ = authClient.login({
      identityProvider: url,
      maxTimeToLive: MAX_TIME_TO_LIVE,
      onSuccess: async () => {
        dispatch(setLoggedIn());
      },
      windowOpenerFeatures:
        `left=${window.screen.width / 2 - 200}, ` +
        `top=${window.screen.height / 2 - 300},` +
        `toolbar=0,location=0,menubar=0,width=400,height=600`
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
