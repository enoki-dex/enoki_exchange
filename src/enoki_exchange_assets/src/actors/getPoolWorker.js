import {Actor, HttpAgent} from "@dfinity/agent";
import {idlFactory, canisterId} from "../../../declarations/enoki_liquidity_pool_worker";

/**
 *
 * @param {import("@dfinity/auth-client").Identity} identity
 * @return {import("@dfinity/agent").ActorSubclass<import("../../../declarations/enoki_liquidity_pool_worker/enoki_liquidity_pool_worker.did.js")._SERVICE>}
 */
const getPoolWorker = (identity) => {
  const agent = new HttpAgent({identity});

  // Fetch root key for certificate validation during development
  if (process.env.NODE_ENV !== "production") {
    agent.fetchRootKey().catch(err => {
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

export default getPoolWorker;
