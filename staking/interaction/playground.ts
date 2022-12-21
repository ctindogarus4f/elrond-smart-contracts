import {
  ABI_PATH,
  CHAIN_ID,
  DECIMALS_SUFFIX,
  EXPLORER,
  REGULAR_GAS_LIMIT,
  OWNER_WALLET,
  PROXY,
  STAKING_SC_ADDRESS,
} from "./config";
import { GREEN, RED, YELLOW } from "./colors";
import {
  AbiRegistry,
  Account,
  Address,
  ArrayVec,
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

const addPackages = async (
  contract: SmartContract,
  owner: Account,
  signer: UserSigner,
  provider: ProxyNetworkProvider,
  watcher: TransactionWatcher,
  resultsParser: ResultsParser,
) => {
  let data = fs.readFileSync("../data/packages.txt", { encoding: "utf8" });
  let lines = data.split(/\r?\n/);

  for (const line of lines) {
    const info = line.split(" ");
    const name = info[0];
    const lock_period_days = info[1];
    const apr_percentage = info[2];
    // remove thousand separator from numbers
    const frequency = info[3].replace(/,/g, "");
    const minStakingAmountWithDecimals = info[4] + DECIMALS_SUFFIX;
    const minStakingAmount = minStakingAmountWithDecimals.replace(/,/g, "");
    const penaltySeconds = info[5].replace(/,/g, "");
    const penaltyFee = info[6];

    let tx = contract.call({
      func: new ContractFunction("addPackage"),
      gasLimit: REGULAR_GAS_LIMIT,
      args: [
        BytesValue.fromUTF8(name),
        new U64Value(lock_period_days),
        new U64Value(apr_percentage),
        new U64Value(frequency),
        new BigUIntValue(minStakingAmount),
        new U64Value(penaltySeconds),
        new U64Value(penaltyFee),
      ],
      chainID: CHAIN_ID,
    });

    console.log(`Adding package ${name}...`);
    tx.setNonce(owner.getNonceThenIncrement());
    await signer.sign(tx);
    await provider.sendTransaction(tx);
    let transactionOnNetwork = await watcher.awaitCompleted(tx);
    let endpointDefinition = contract.getEndpoint("addPackage");
    let { returnCode, returnMessage } = resultsParser.parseOutcome(
      transactionOnNetwork,
      endpointDefinition,
    );

    if (returnCode.isSuccess()) {
      console.log(
        GREEN,
        `SUCCESS! Package added: ${name}, tx hash: ${EXPLORER}/transactions/${tx.getHash()}.`,
      );
    } else {
      console.log(
        RED,
        `ERROR! tx hash: ${EXPLORER}/transactions/${tx.getHash()}, tx details: ${returnMessage}.`,
      );
    }

    console.log(`Fetching package ${name}...`);
    let query = contract.createQuery({
      func: new ContractFunction("getPackageInfo"),
      args: [BytesValue.fromUTF8(name)],
    });
    let queryResponse = await provider.queryContract(query);
    endpointDefinition = contract.getEndpoint("getPackageInfo");
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

const main = async () => {
  // ----------------------- CODEC SETUP -----------------------
  let jsonContent: string = fs.readFileSync(ABI_PATH, { encoding: "utf8" });
  let json = JSON.parse(jsonContent);
  let abiRegistry = AbiRegistry.create(json);
  let abi = new SmartContractAbi(abiRegistry, ["StakingContract"]);
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
    address: new Address(STAKING_SC_ADDRESS),
    abi: abi,
  });
  // ------------------------ SC SETUP -------------------------

  // ---------------------- CONTRACT CHECK ---------------------
  {
    console.log("Staking SC address:");
    console.log(YELLOW, STAKING_SC_ADDRESS, "\n");

    {
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

    {
      console.log("Fetching package names...");
      let query = contract.createQuery({
        func: new ContractFunction("getPackageNames"),
      });
      let queryResponse = await provider.queryContract(query);
      let endpointDefinition = contract.getEndpoint("getPackageNames");
      let { firstValue } = resultsParser.parseQueryResponse(
        queryResponse,
        endpointDefinition,
      );
      let decodedResponse = (<ArrayVec>firstValue).valueOf();
      for (const packageName of decodedResponse) {
        console.log(YELLOW, packageName, "\n");
      }
    }
  }

  // ---------------------- CONTRACT CHECK ---------------------

  await addPackages(contract, owner, signer, provider, watcher, resultsParser);
};

(async () => {
  await main();
})();
