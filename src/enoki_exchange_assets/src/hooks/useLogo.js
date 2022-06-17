import React from "react";
import {createActor} from "../../../declarations/enoki_wrapped_token";

const cache = {};

/**
 *
 * @return String
 */
const useLogo = ({canisterId}) => {
  const [logo, setLogo] = React.useState(cache[canisterId] || '');

  React.useEffect(() => {
    if (logo) return;

    let stop = false;

    createActor(canisterId)
      .getLogo()
      .then(logo => {
        cache[canisterId] = logo;
        if (stop) return;
        setLogo(logo);
      });

    return () => {
      stop = true;
    }
  }, [canisterId]);

  return logo;
}

export default useLogo;
