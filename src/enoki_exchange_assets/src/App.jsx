import * as React from 'react';
import {enoki_exchange} from "../../declarations/enoki_exchange";

const App = () => {
  const [greeting, setGreeting] = React.useState("");
  const [pending, setPending] = React.useState(false);
  const inputRef = React.useRef();

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (pending) return;
    setPending(true);

    // Interact with hello actor, calling the greet method
    const response = await enoki_exchange.getOwner();
    setGreeting(`the owner is ${response.toString()}`);
    setPending(false);
    return false;
  }

  return (
    <main>
      <img src="logo.png" alt="DFINITY logo" />
      <form onSubmit={handleSubmit}>
        <label htmlFor="name">get exchange owner: &nbsp;</label>
        <button id="clickMeBtn" type="submit" disabled={pending}>Click Me!</button>
      </form>
      <section id="greeting">{greeting}</section>
    </main>
  )
}

export default App;
