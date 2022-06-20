import {exec} from 'child_process';

const command_internal = command => new Promise((resolve, reject) => {
  exec(command, ((error, stdout) => {
    if (error) {
      console.log(`[dfx] ERROR with ${command} =>`, error);
      return reject(error);
    }
    let response = stdout.trimEnd();
    console.log(`[dfx] ${command} => ${response}`);
    return resolve(response);
  }));
});

class Dfx {
  constructor(network) {
    this.network = network;
  }

  async exec(command, subcommand, ...args) {
    let commandStr = `dfx ${command} --network ${this.network} ${subcommand}`;
    if (args.length) {
      commandStr +=
        " '(" +
        args.map(arg => {
          if (arg.principal) {
            return principalArgument(arg.principal);
          } else if (arg.raw) {
            return arg.raw;
          } else if (arg.string) {
            return `"${arg.string}"`;
          } else if (arg.nat) {
            return `${(arg.nat).toString()} : nat`;
          } else {
            let argStr;
            try {
              argStr = JSON.stringify(arg)
            } catch (e) {
              argStr = arg
            }
            throw new Error(`invalid argument: ${argStr}`);
          }
        }).join(', ') +
        ")'";
    }
    return await command_internal(commandStr);
  }
}

export const createClient = ({network}) => new Dfx(network);

export const principalArgument = principal => `principal "${principal}"`;
