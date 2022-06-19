import React from "react";
import useLogin from "./useLogin";
import getEnokiExchange from "../actors/getEnokiExchange";

// TODO: remove this hook when exchange updates automatically
const useHeartbeat = () => {
  const {
    isLoggedIn, getIdentity
  } = useLogin();
  const [lastUpdateTime, setLastUpdateTime] = React.useState(null);

  React.useEffect(() => {
    if (!isLoggedIn) {
      return;
    }
    let stop = false;

    const wait = delay => new Promise(resolve => setTimeout(resolve, delay));
    const run = async () => {
      while (!stop) {
        try {
          await getEnokiExchange(getIdentity()).triggerRun();
          if (stop) return;
          setLastUpdateTime(Date.now());
        } catch (err) {
          console.error("error with exchange heartbeat: ", err);
        }
        await wait(5000);
      }
    }
    run()
      .catch(e => console.error("error with heartbeat: ", e));

    return () => {
      stop = true;
    }
  }, [isLoggedIn])


  return lastUpdateTime;
}

export default useHeartbeat;
