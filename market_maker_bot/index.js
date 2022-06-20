import App from "./app.js";

let network = 'local';

let customFunction = null;
let varSet = null;
process.argv.slice(2).forEach(arg => {
  if (/^--/.test(arg)) {
    varSet = arg;
  } else if (varSet) {
    switch (varSet) {
      case "--network":
        if (arg === 'ic' || arg === 'local') {
          network = arg;
        } else {
          throw new Error(`invalid network '${arg}'`);
        }
        break;
      default:
        throw new Error(`unknown variable '${varSet}'`);
    }
    varSet = null;
  } else {
    switch (arg) {
      case 'mint':
        customFunction = arg;
        break;
      default:
        throw new Error(`unknown method '${arg}'`);
    }
  }
})

console.log('using network: ', network);

const app = new App(network);

if (customFunction === 'mint') {
  app.mint()
    .then(() => console.log('done minting'))
    .catch(e => console.error('error minting: ', e));
} else {
  process.on('SIGINT', function () {
    console.log(" Cleaning up...");
    app.exit();
  });


  app.run()
    .then(() => console.log("exited gracefully"))
    .catch(err => console.error(`APP FAILED with error:`, err));
}
