import { Actor, HttpAgent } from "@dfinity/agent";
import {idlFactory} from "../../../declarations/enoki_wrapped_token";

/**
 *
 * @param {import("@dfinity/auth-client").Identity} identity
 * @param {string | import("@dfinity/principal").Principal} canisterId Canister ID of Agent
 * @return {import("@dfinity/agent").ActorSubclass<import("../../../declarations/enoki_wrapped_token/enoki_wrapped_token.did.js")._SERVICE>}
 */
const getMainToken = (identity, canisterId) => {
  const agent = new HttpAgent({identity});

  // Fetch root key for certificate validation during development
  if(process.env.NODE_ENV !== "production") {
    agent.fetchRootKey().catch(err=>{
      console.warn("Unable to fetch root key. Check to ensure that your local replica is running");
      console.error(err);
    });
  }

  // Creates an actor with using the candid interface and the HttpAgent
  return Actor.createActor(idlFactory, {
    agent,
    canisterId,
  });
};

export default getMainToken;
