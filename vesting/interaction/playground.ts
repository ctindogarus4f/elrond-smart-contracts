import {
  ABI_PATH,
  DECIMALS_SUFFIX,
  EXPLORER,
  GAS_LIMIT,
  OWNER_WALLET,
  PROXY,
  VESTING_SC_ADDRESS,
} from "./config";
import { GREEN, RED, YELLOW } from "./colors";
import {
  AbiRegistry,
  Account,
  Address,
  AddressType,
  AddressValue,
  BigUIntValue,
  BinaryCodec,
  BooleanValue,
  BytesValue,
  ContractFunction,
  GasLimit,
  ListType,
  NetworkConfig,
  ProxyProvider,
  SmartContract,
  TokenIdentifierType,
  U8Value,
  U64Value,
  U64Type,
  UserSigner,
} from "@elrondnetwork/erdjs";
import axios from "axios";
const fs = require("fs");

const addGroups = async (
  contract: SmartContract,
  owner: Account,
  signer: UserSigner,
  provider: ProxyProvider,
  abi: AbiRegistry,
  codec: BinaryCodec,
) => {
  let groupInfoType = abi.getStruct("GroupInfo");

  let data = fs.readFileSync("../data/groups.txt", { encoding: "utf8" });
  let lines = data.split(/\r?\n/);

  for (const line of lines) {
    const info = line.split(" ");
    const name = info[0];
    // remove thousand separator from numbers
    const maxAllocationWithDecimals = info[1] + DECIMALS_SUFFIX;
    const maxAllocation = maxAllocationWithDecimals.replace(/,/g, "");
    const cliff = info[2].replace(/,/g, "");
    const frequency = info[3].replace(/,/g, "");
    const percentage = info[4];

    let tx = contract.call({
      func: new ContractFunction("addGroup"),
      gasLimit: new GasLimit(GAS_LIMIT),
      args: [
        BytesValue.fromUTF8(name),
        new BigUIntValue(maxAllocation),
        new U64Value(cliff),
        new U64Value(frequency),
        new U8Value(percentage),
      ],
    });

    console.log(`Adding group ${name}...`);
    tx.setNonce(owner.nonce);
    owner.incrementNonce();
    await signer.sign(tx);
    await tx.send(provider);
    await tx.awaitExecuted(provider);

    let result = await getTransaction(`${tx.getHash()}`);
    if (result.success) {
      console.log(
        GREEN,
        `SUCCESS! Group added: ${name}, tx hash: ${EXPLORER}/transactions/${tx.getHash()}.`,
      );
    } else {
      console.log(
        RED,
        `ERROR! tx hash: ${EXPLORER}/transactions/${tx.getHash()}, tx details: ${
          result.errorMessage
        }.`,
      );
    }

    console.log(`Fetching group ${name}...`);
    let response = await contract.runQuery(provider, {
      func: new ContractFunction("getGroupInfo"),
      args: [BytesValue.fromUTF8(name)],
    });

    let decodedResponse = codec
      .decodeTopLevel(response.outputUntyped()[0], groupInfoType)
      .valueOf();
    Object.keys(decodedResponse).forEach(key => {
      decodedResponse[key] = decodedResponse[key].toString();
    });
    console.log(YELLOW, decodedResponse, "\n");
  }
};

const addBeneficiaries = async (
  contract: SmartContract,
  owner: Account,
  signer: UserSigner,
  provider: ProxyProvider,
  abi: AbiRegistry,
  codec: BinaryCodec,
) => {
  let beneficiaryInfoType = abi.getStruct("BeneficiaryInfo");

  let data = fs.readFileSync("../data/beneficiaries.txt", { encoding: "utf8" });
  let lines = data.split(/\r?\n/);

  for (const line of lines) {
    const info = line.split(" ");
    const addr = info[0];
    const addrObj = new Address(addr);
    const canBeRevoked = info[1] === "temporary";
    const groupName = info[2];
    // remove thousand separator from numbers
    const startTimestamp = info[3].replace(/,/g, "");
    const tokensAllocatedWithDecimals = info[4] + DECIMALS_SUFFIX;
    const tokensAllocated = tokensAllocatedWithDecimals.replace(/,/g, "");

    let tx = contract.call({
      func: new ContractFunction("addBeneficiary"),
      gasLimit: new GasLimit(GAS_LIMIT),
      args: [
        new AddressValue(addrObj),
        new BooleanValue(canBeRevoked),
        BytesValue.fromUTF8(groupName),
        new U64Value(startTimestamp),
        new BigUIntValue(tokensAllocated),
      ],
    });

    console.log(`Adding beneficiary ${addr}...`);
    tx.setNonce(owner.nonce);
    owner.incrementNonce();
    await signer.sign(tx);
    await tx.send(provider);
    await tx.awaitExecuted(provider);

    let result = await getTransaction(`${tx.getHash()}`);
    if (result.success) {
      console.log(
        GREEN,
        `SUCCESS! Beneficiary added: ${addr}, tx hash: ${EXPLORER}/transactions/${tx.getHash()}.`,
      );
    } else {
      console.log(
        RED,
        `ERROR! tx hash: ${EXPLORER}/transactions/${tx.getHash()}, tx details: ${
          result.errorMessage
        }.`,
      );
    }

    console.log(`Fetching ids for beneficiary ${addr}...`);
    let response = await contract.runQuery(provider, {
      func: new ContractFunction("getBeneficiaryIds"),
      args: [new AddressValue(addrObj)],
    });

    let beneficiaryIds = codec
      .decodeTopLevel(response.outputUntyped()[0], new ListType(new U64Type()))
      .valueOf();
    for (const beneficiaryId of beneficiaryIds) {
      console.log(YELLOW, beneficiaryId, "\n");
    }
  }
};

const getTransaction = async (txHash: string) => {
  const { data } = await axios.get(`${PROXY}/transactions/${txHash}`, {
    timeout: 4000,
  });

  const success = data.status === "success";
  return {
    success,
    errorMessage: success ? "" : data.operations[0].message,
  };
};

const main = async () => {
  // ----------------------- CODEC SETUP -----------------------
  let abi = await AbiRegistry.load({ files: [ABI_PATH] });
  let codec = new BinaryCodec();
  // ----------------------- CODEC SETUP -----------------------

  // ---------------------- NETWORK SETUP ----------------------
  const provider = new ProxyProvider(PROXY, { timeout: 4000 });
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

  // ---------------------- CONTRACT CHECK ---------------------
  {
    console.log("Vesting SC address:");
    console.log(YELLOW, VESTING_SC_ADDRESS, "\n");

    console.log("Getting token identifier...");
    let response = await contract.runQuery(provider, {
      func: new ContractFunction("getTokenIdentifier"),
    });

    let decodedResponse = codec
      .decodeTopLevel(response.outputUntyped()[0], new TokenIdentifierType())
      .valueOf()
      .toString();
    console.log(YELLOW, decodedResponse, "\n");
  }

  {
    console.log("Getting multisig address...");
    let response = await contract.runQuery(provider, {
      func: new ContractFunction("getMultisigAddress"),
    });

    let decodedResponse = codec
      .decodeTopLevel(response.outputUntyped()[0], new AddressType())
      .valueOf()
      .bech32();
    console.log(YELLOW, decodedResponse, "\n");
  }
  // ---------------------- CONTRACT CHECK ---------------------

  const action = process.argv.length > 2 ? process.argv[2] : undefined;
  const shouldAddGroups = action === undefined || action === "add_groups";
  const shouldAddBeneficiaries =
    action === undefined || action === "add_beneficiaries";

  // ------------------------ ADD GROUPS -----------------------
  if (shouldAddGroups) {
    await addGroups(contract, owner, signer, provider, abi, codec);
  }
  // ------------------------ ADD GROUPS -----------------------

  // -------------------- ADD BENEFICIARIES --------------------
  if (shouldAddBeneficiaries) {
    await addBeneficiaries(contract, owner, signer, provider, abi, codec);
  }
  // -------------------- ADD BENEFICIARIES --------------------
};

(async () => {
  await main();
})();
