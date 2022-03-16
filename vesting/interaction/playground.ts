import {
  ABI_PATH,
  GAS_LIMIT,
  OWNER_WALLET,
  PROXY,
  VESTING_SC_ADDRESS,
} from "./config";
import {
  AbiRegistry,
  Account,
  Address,
  AddressValue,
  BigUIntValue,
  BinaryCodec,
  BooleanValue,
  UserSigner,
  ContractFunction,
  GasLimit,
  NetworkConfig,
  ProxyProvider,
  SmartContract,
  TransactionWatcher,
  U8Value,
  U64Value,
} from "@elrondnetwork/erdjs";
const fs = require("fs");

const main = async () => {
  // ----------------------- CODEC SETUP -----------------------
  let abi = await AbiRegistry.load({ files: [ABI_PATH] });
  let groupInfoType = abi.getStruct("GroupInfo");
  let beneficiaryInfoType = abi.getStruct("BeneficiaryInfo");
  let codec = new BinaryCodec();
  // ----------------------- CODEC SETUP -----------------------

  // ---------------------- NETWORK SETUP ----------------------
  const provider = new ProxyProvider(PROXY);
  await NetworkConfig.getDefault().sync(provider);
  // ---------------------- NETWORK SETUP ----------------------

  // ----------------- SIGNER AND OWNER SETUP ------------------
  const privateKey = fs.readFileSync(OWNER_WALLET, { encoding: "utf8" });
  const signer = UserSigner.fromPem(privateKey);
  const owner = new Account(signer.getAddress());
  await owner.sync(provider);
  // ----------------- SIGNER AND OWNER SETUP ------------------

  // ------------------------ SC SETUP -------------------------
  let contract = new SmartContract({
    address: new Address(VESTING_SC_ADDRESS),
  });
  // ------------------------ SC SETUP -------------------------

  // ------------------------ ADD GROUPS -----------------------
  let data = fs.readFileSync("../data/groups.txt", { encoding: "utf8" });
  let lines = data.split(/\r?\n/);

  for (const line of lines) {
    const info = line.split(" ");
    const name = info[0];
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

    console.log(`# Adding group ${name}...`);
    tx.setNonce(owner.nonce);
    owner.incrementNonce();
    await signer.sign(tx);
    await tx.send(provider);

    let watcher = new TransactionWatcher(
      tx.getHash(),
      provider,
      TransactionWatcher.DefaultPollingInterval,
      TransactionWatcher.DefaultTimeout * 3,
    );
    await watcher.awaitStatus(
      status => status.isExecuted(),
      TransactionWatcher.NoopOnStatusReceived,
    );
    let wrappedResult = await tx.getAsOnNetwork(
      provider,
      undefined,
      false,
      true,
    );
    let result = wrappedResult.getSmartContractResults().getImmediate();
    if (result.isSuccess()) {
      console.log(`# SUCCESS! Group added: ${name}, tx hash: ${tx.getHash()}.`);
    } else {
      console.log(
        `# ERROR! tx hash: ${tx.getHash()}, tx details: ${result.getReturnMessage()}.`,
      );
    }

    console.log(`# Fetching group ${name}...`);
    let response = await contract.runQuery(provider, {
      func: new ContractFunction("getGroupInfo"),
      args: [new U8Value(id)],
    });

    let decodedResponse = codec
      .decodeTopLevel(response.outputUntyped()[0], groupInfoType)
      .valueOf();
    console.log(decodedResponse, "\n");
  }
  // ------------------------ ADD GROUPS -----------------------

  // ------------------------ ADD BENEFICIARIES -----------------------
  data = fs.readFileSync("../data/beneficiaries.txt", { encoding: "utf8" });
  lines = data.split(/\r?\n/);

  for (const line of lines) {
    const info = line.split(" ");
    const addr = info[0];
    const addrObj = new Address(addr);
    const canBeRevoked = info[1] === "temporary";
    const groupId = info[2];
    // remove thousand separator from numbers
    const startTimestamp = info[3].replace(/,/g, "");
    const tokensAllocated = info[4].replace(/,/g, "");

    let tx = contract.call({
      func: new ContractFunction("addBeneficiary"),
      gasLimit: new GasLimit(GAS_LIMIT),
      args: [
        new AddressValue(addrObj),
        new BooleanValue(canBeRevoked),
        new U8Value(groupId),
        new U64Value(startTimestamp),
        new BigUIntValue(tokensAllocated),
      ],
    });

    console.log(`# Adding beneficiary ${addr}...`);
    tx.setNonce(owner.nonce);
    owner.incrementNonce();
    await signer.sign(tx);
    await tx.send(provider);

    let watcher = new TransactionWatcher(
      tx.getHash(),
      provider,
      TransactionWatcher.DefaultPollingInterval,
      TransactionWatcher.DefaultTimeout * 3,
    );
    await watcher.awaitStatus(
      status => status.isExecuted(),
      TransactionWatcher.NoopOnStatusReceived,
    );
    let wrappedResult = await tx.getAsOnNetwork(
      provider,
      undefined,
      false,
      true,
    );
    let result = wrappedResult.getSmartContractResults().getImmediate();
    if (result.isSuccess()) {
      console.log(`# SUCCESS! Group added: ${addr}, tx hash: ${tx.getHash()}.`);
    } else {
      console.log(
        `# ERROR! tx hash: ${tx.getHash()}, tx details: ${result.getReturnMessage()}.`,
      );
    }

    console.log(`# Fetching beneficiary ${addr}...`);
    let response = await contract.runQuery(provider, {
      func: new ContractFunction("getBeneficiaryInfo"),
      args: [new AddressValue(addrObj)],
    });

    let decodedResponse = codec
      .decodeTopLevel(response.outputUntyped()[0], beneficiaryInfoType)
      .valueOf();
    console.log(decodedResponse, "\n");
  }
  // ------------------------ ADD BENEFICIARIES -----------------------
};

(async () => {
  await main();
})();
