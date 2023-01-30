import {
  ABI_PATH,
  CHAIN_ID,
  DECIMALS_SUFFIX,
  EXPLORER,
  REGULAR_GAS_LIMIT,
  OWNER_WALLET,
  PROXY,
  VESTING_SC_ADDRESS,
} from "./config";
import { GREEN, RED, YELLOW } from "./colors";
import {
  AbiRegistry,
  Account,
  Address,
  AddressValue,
  BigUIntValue,
  BytesValue,
  ContractFunction,
  ResultsParser,
  SmartContract,
  SmartContractAbi,
  Struct,
  TokenIdentifierValue,
  TransactionWatcher,
  U64Value,
} from "@elrondnetwork/erdjs";
import { UserSigner } from "@elrondnetwork/erdjs-walletcore";
import { ProxyNetworkProvider } from "@elrondnetwork/erdjs-network-providers";

const fs = require("fs");

const addGroups = async (
  contract: SmartContract,
  owner: Account,
  signer: UserSigner,
  provider: ProxyNetworkProvider,
  watcher: TransactionWatcher,
  resultsParser: ResultsParser,
) => {
  let data = fs.readFileSync("../data/groups.txt", { encoding: "utf8" });
  let lines = data.split(/\r?\n/);

  for (const line of lines) {
    const info = line.split(" ");
    const name = info[0];
    // remove thousand separator from numbers
    const frequency = info[1].replace(/,/g, "");
    const lockPeriod = info[2].replace(/,/g, "");

    let tx = contract.call({
      func: new ContractFunction("addGroup"),
      gasLimit: REGULAR_GAS_LIMIT,
      args: [
        BytesValue.fromUTF8(name),
        new U64Value(frequency),
        new U64Value(lockPeriod),
      ],
      chainID: CHAIN_ID,
    });

    console.log(`Adding group ${name}...`);
    tx.setNonce(owner.getNonceThenIncrement());
    await signer.sign(tx);
    await provider.sendTransaction(tx);
    let transactionOnNetwork = await watcher.awaitCompleted(tx);
    let endpointDefinition = contract.getEndpoint("addGroup");
    let { returnCode, returnMessage } = resultsParser.parseOutcome(
      transactionOnNetwork,
      endpointDefinition,
    );

    if (returnCode.isSuccess()) {
      console.log(
        GREEN,
        `SUCCESS! Group added: ${name}, tx hash: ${EXPLORER}/transactions/${tx.getHash()}.`,
      );
    } else {
      console.log(
        RED,
        `ERROR! tx hash: ${EXPLORER}/transactions/${tx.getHash()}, tx details: ${returnMessage}.`,
      );
    }

    console.log(`Fetching group ${name}...`);
    let query = contract.createQuery({
      func: new ContractFunction("getGroupInfo"),
      args: [BytesValue.fromUTF8(name)],
    });
    let queryResponse = await provider.queryContract(query);
    endpointDefinition = contract.getEndpoint("getGroupInfo");
    let { firstValue } = resultsParser.parseQueryResponse(
      queryResponse,
      endpointDefinition,
    );
    let decodedResponse = (<Struct>firstValue).valueOf();
    Object.keys(decodedResponse).forEach(key => {
      decodedResponse[key] = decodedResponse[key].toString();
    });

    console.log(YELLOW, decodedResponse, "\n");
  }
};

const addBeneficiariesHelper = async (
  contract: SmartContract,
  owner: Account,
  signer: UserSigner,
  provider: ProxyNetworkProvider,
  watcher: TransactionWatcher,
  resultsParser: ResultsParser,
  args: any,
) => {
  let tx = contract.call({
    func: new ContractFunction("addMultipleBeneficiaries"),
    gasLimit: REGULAR_GAS_LIMIT,
    args: args,
    chainID: CHAIN_ID,
  });

  console.log(`Adding beneficiaries...`);
  tx.setNonce(owner.getNonceThenIncrement());
  await signer.sign(tx);
  await provider.sendTransaction(tx);
  let transactionOnNetwork = await watcher.awaitCompleted(tx);
  let endpointDefinition = contract.getEndpoint("addMultipleBeneficiaries");
  let { returnCode, returnMessage } = resultsParser.parseOutcome(
    transactionOnNetwork,
    endpointDefinition,
  );

  if (returnCode.isSuccess()) {
    console.log(
      GREEN,
      `SUCCESS! Beneficiaries added, tx hash: ${EXPLORER}/transactions/${tx.getHash()}.`,
    );
  } else {
    console.log(
      RED,
      `ERROR! tx hash: ${EXPLORER}/transactions/${tx.getHash()}, tx details: ${returnMessage}.`,
    );
  }
};

const addBeneficiaries = async (
  contract: SmartContract,
  owner: Account,
  signer: UserSigner,
  provider: ProxyNetworkProvider,
  watcher: TransactionWatcher,
  resultsParser: ResultsParser,
) => {
  let data = fs.readFileSync("../data/beneficiaries.txt", { encoding: "utf8" });
  let lines = data.split(/\r?\n/);

  let idx = 0;
  let args: any = [];
  let groupName;
  let startTimestamp;
  for (const line of lines) {
    const info = line.split(" ");
    const addr = info[0];
    const addrObj = new Address(addr);
    groupName = info[1];
    // remove thousand separator from numbers
    startTimestamp = info[2].replace(/,/g, "");
    const tokensAllocatedWithDecimals = info[3] + DECIMALS_SUFFIX;
    const tokensAllocated = tokensAllocatedWithDecimals.replace(/,/g, "");

    args.push(new AddressValue(addrObj));
    args.push(new BigUIntValue(tokensAllocated));

    idx++;
    if (idx % 50 !== 0) {
      continue;
    }

    args = [
      BytesValue.fromUTF8(groupName),
      new U64Value(startTimestamp),
      ...args,
    ];

    await addBeneficiariesHelper(
      contract,
      owner,
      signer,
      provider,
      watcher,
      resultsParser,
      args,
    );

    args = [];
  }

  if (args) {
    args = [
      BytesValue.fromUTF8(groupName),
      new U64Value(startTimestamp),
      ...args,
    ];

    await addBeneficiariesHelper(
      contract,
      owner,
      signer,
      provider,
      watcher,
      resultsParser,
      args,
    );
  }
};

const main = async () => {
  // ----------------------- CODEC SETUP -----------------------
  let jsonContent: string = fs.readFileSync(ABI_PATH, { encoding: "utf8" });
  let json = JSON.parse(jsonContent);
  let abiRegistry = AbiRegistry.create(json);
  let abi = new SmartContractAbi(abiRegistry, ["VestingContract"]);
  let resultsParser = new ResultsParser();
  // ----------------------- CODEC SETUP -----------------------

  // ---------------------- NETWORK SETUP ----------------------
  const provider = new ProxyNetworkProvider(PROXY, { timeout: 4000 });
  const watcher = new TransactionWatcher(provider);
  // ---------------------- NETWORK SETUP ----------------------

  // ----------------- SIGNER AND OWNER SETUP ------------------
  const privateKey = fs.readFileSync(OWNER_WALLET, { encoding: "utf8" });
  const signer = UserSigner.fromPem(privateKey);
  const owner = new Account(signer.getAddress());
  const ownerOnNetwork = await provider.getAccount(owner.address);
  owner.update(ownerOnNetwork);

  // ----------------- SIGNER AND OWNER SETUP ------------------

  // ------------------------ SC SETUP -------------------------
  let contract = new SmartContract({
    address: new Address(VESTING_SC_ADDRESS),
    abi: abi,
  });
  // ------------------------ SC SETUP -------------------------

  // ---------------------- CONTRACT CHECK ---------------------
  {
    console.log("Vesting SC address:");
    console.log(YELLOW, VESTING_SC_ADDRESS, "\n");

    console.log("Getting token identifier...");
    let query = contract.createQuery({
      func: new ContractFunction("getTokenIdentifier"),
    });
    let queryResponse = await provider.queryContract(query);
    let endpointDefinition = contract.getEndpoint("getTokenIdentifier");
    let { firstValue } = resultsParser.parseQueryResponse(
      queryResponse,
      endpointDefinition,
    );

    let decodedResponse = <TokenIdentifierValue>firstValue;
    console.log(YELLOW, decodedResponse, "\n");
  }

  // ---------------------- CONTRACT CHECK ---------------------

  const action = process.argv.length > 2 ? process.argv[2] : undefined;
  const shouldAddGroups = action === undefined || action === "add_groups";
  const shouldAddBeneficiaries =
    action === undefined || action === "add_beneficiaries";

  // ------------------------ ADD GROUPS -----------------------
  if (shouldAddGroups) {
    await addGroups(contract, owner, signer, provider, watcher, resultsParser);
  }
  // ------------------------ ADD GROUPS -----------------------

  // -------------------- ADD BENEFICIARIES --------------------
  if (shouldAddBeneficiaries) {
    await addBeneficiaries(
      contract,
      owner,
      signer,
      provider,
      watcher,
      resultsParser,
    );
  }
  // -------------------- ADD BENEFICIARIES --------------------
};

(async () => {
  await main();
})();
