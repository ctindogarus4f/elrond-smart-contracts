import {
  Account,
  Address,
  UserSigner,
  ContractFunction,
  GasLimit,
  NetworkConfig,
  ProxyProvider,
  SmartContract,
  U8Value,
  U64Value,
} from "@elrondnetwork/erdjs";
const fs = require("fs");

const GAS_LIMIT = 600000000;
const OWNER_WALLET = "/Users/constantindogaru/elrond-wallet/wallet-owner.pem";
const PROXY = "https://devnet-api.elrond.com";
const VESTING_SC_ADDRESS =
  "erd1qqqqqqqqqqqqqpgqfh8txltrewpf6rrec7ntzfw682gdy98d8x5qcq49t2";

const main = async () => {
  // NETWORK SETUP
  const provider = new ProxyProvider(PROXY);
  await NetworkConfig.getDefault().sync(provider);

  // SIGNER AND OWNER SETUP
  const privateKey = fs.readFileSync(OWNER_WALLET, { encoding: "utf8" });
  const signer = UserSigner.fromPem(privateKey);
  const owner = new Account(signer.getAddress());
  await owner.sync(provider);

  // SC SETUP
  let contract = new SmartContract({
    address: new Address(VESTING_SC_ADDRESS),
  });

  const data = fs.readFileSync("groups_data.txt", { encoding: "utf8" });
  const lines = data.split(/\r?\n/);

  for (const line of lines) {
    const info = line.split(" ");
    const id = info[1];
    // remove thousand separator from numbers
    const cliff = info[2].replace(/,/g, "");
    const duration = info[3].replace(/,/g, "");
    const percentage = info[4];

    let tx = contract.call({
      func: new ContractFunction("addGroup"),
      gasLimit: new GasLimit(GAS_LIMIT),
      args: [
        new U8Value(id),
        new U64Value(cliff),
        new U64Value(duration),
        new U8Value(percentage),
      ],
    });

    tx.setNonce(owner.nonce);
    await signer.sign(tx);
    await tx.send(provider);
  }
};

(async () => {
  await main();
})();
