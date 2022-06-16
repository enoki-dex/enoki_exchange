import {Actor, HttpAgent} from "@dfinity/agent";
import {idlFactory, canisterId} from "../../../declarations/enoki_exchange";
import getTokenShard from "./getTokenShard";
import getMainToken from "./getMainToken";
import getEnokiBroker from "./getEnokiBroker";

/**
 *
 * @param {import("@dfinity/auth-client").Identity} identity
 * @return {import("@dfinity/agent").ActorSubclass<import("../../../declarations/enoki_exchange/enoki_exchange.did.js")._SERVICE>}
 */
const getEnokiExchange = (identity) => {
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

export default getEnokiExchange;

/**
 *
 * @return {Promise<import("@dfinity/agent").ActorSubclass<import("../../../declarations/enoki_broker_1/enoki_broker_1.did.js")._SERVICE>>}
 */
export const getAssignedBroker = async (identity) => {
  let assigned_broker;
  try {
    assigned_broker = await getEnokiExchange(identity).getAssignedBroker(identity.getPrincipal());
  } catch (err) {
    assigned_broker = await getEnokiExchange(identity).register(identity.getPrincipal());
  }
  return getEnokiBroker(identity, assigned_broker);
}
