import { NetworkConfig, ProxyProvider } from "@elrondnetwork/erdjs";

async function main() {
  let provider = new ProxyProvider("https://devnet-api.elrond.com");
  await NetworkConfig.getDefault().sync(provider);

  console.log(NetworkConfig.getDefault().MinGasPrice);
  console.log(NetworkConfig.getDefault().ChainID);
}

(async () => {
  await main();
})();
