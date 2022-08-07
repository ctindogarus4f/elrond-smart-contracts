# How to use the script

1. Define your own `beneficiaries` and `groups` inside `../data`.
2. Change `config.ts` parameters. (`VESTING_SC_ADDRESS` should be replaced by your own SC address, `OWNER_WALLET` should be replaced by your pem file. `EXPLORER`, `PROXY` and `CHAIN_ID` also need to change if you're not deploying on testnet)
3. Run the script:
- npm run start - to add groups + beneficiaries
- npm run add_groups - to add groups
- npm run add_beneficiaries - to add beneficiaries
