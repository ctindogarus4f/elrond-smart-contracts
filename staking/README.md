## Installation

Clone this repository:  
`git clone https://github.com/S4F-IT/elrond-smart-contracts`

Install dependencies:  
`cd staking && erdpy contract build`

## Tests

For UNIT tests, run:  
`erdpy contract test`

## Deployment

1. Access https://testnet-wallet.elrond.com/issue-token and issue your own Fungible ESDT token.
2. Edit `erdpy.json` and make sure that:
- `arguments` parameter points to the token that you've created in step 1
- `pem` parameter points to your pem file

To deploy the staking contract, run:  
`erdpy contract deploy`

After you've deployed the smart contract, you need to transfer some amount of the Fungible ESDT token from your own wallet to the SC address. Use https://testnet-wallet.elrond.com/tokens for the transfer.

By default, the smart contract will get deployed on `testnet`. If you need other deployment configuration, please change `erdpy.json`:
- proxy (ex: `https://devnet-api.elrond.com` for devnet)
- chainID (ex: `D` for devnet)
- you also need to create the token on devnet (https://devnet-wallet.elrond.com/issue-token) and transfer it to the SC address via https://devnet-wallet.elrond.com/tokens.

## Setup

1. You first deploy your own Fungible ESDT token.
2. You deploy and initiate the smart contract with your own Fungible ESDT token.
3. You transfer some amount of the Fungible ESDT token to the SC address.
4. You define some packages via `addPackage` calls (ex: bronze package, silver package, gold package)

## Implementation details

All the packages are saved as a key/value map inside the storage. The key represents the package name (bronze, silver, gold etc.) and the value represents a structure with details about that package (lock_period, apr_percentage, rewards_frequency etc.).  
Example:
```
team -> {enabled: true, lock_period: 180, apr_percentage: 27, rewards_frequency: 1, min_stake_amount: 30000, total_staked_amunt: 0}
```

All the stakers are saved as a key/value map inside the storage. The key represents the staker wallet address and the value represents an array of ids. Each id represents a different stake for that staker (ex: if the user has staked tokens twice in our platform, he will have 2 stake ids). For fetching more information about the stake id, each id is stored in a different key/value map. The key represents the id and the value represents a structure with information about that specific id).
Example:
```
erd1z72y3rwuz2qga43m7evtexgy0vkvv8g9uukvz4emfngxara4n94qs0ruxp -> [5, 6]
5 -> {package_name: silver, stake_timestamp: 1648587600, tokens_staked: 5000, last_claim_of_rewards: 1648690000}
6 -> {package_name: silver, stake_timestamp: 1648512300, tokens_staked: 10000, last_claim_of_rewards: 1648710000}
```

## Available methods that can be called

***Read methods***
1) `getTotalTokensStaked()`: Get the total amount of tokens that are staked (it includes all the packages).
2) `getTokenIdentifier()`: The token that is used for staking.
3) `getPausedStake()`: True if the staking is paused, false otherwise.
4) `getPausedRewardsTimestamp()`: 0 if the rewards are not paused, unix timestamp otherwise.
5) `getStakerCounter()`: The total number of ids.
6) `getStakerInfo(id)`: The details about a specific id.
7) `getStakerIds(staker)`: An array with ids for a specific staker wallet.
8) `getPackageInfo(package_name)`: Get all the information about a specific package.

***Write methods***
1) `pauseStake()`: Temporarily pause stake. Can be called only by the owner of the SC.
2) `unpauseStake()`: Unpause stake. Can be called only by the owner of the SC.
3) `pauseRewards()`: Temporarily pause rewards. Can be called only by the owner of the SC.
4) `unpauseRewards()`: Unpause rewards. Can be called only by the owner of the SC.
5) `addPackage(package_name, lock_period, apr_percentage, rewards_frequency, min_stake_amount)`: Define a new package inside the SC. Can be called only by the owner of the SC.
6) `enablePackage(package_name)` - Enable a package. Can be called only by the owner of the SC.
7) `disablePackage(package_name)` - Temporarily disable a package (new stakes in that package will be forbidden). Can be called only by the owner of the SC.
8) `createNewStake(package_name)` - Define a new stake inside the SC for a given package. Can be called only by anyone.
9) `reinvestRewardsToExistingStake(id)` - Reinvest rewards from a stake id to the same stake id. Used if you want to get a compound interest. Can be called only by the staker of that stake id.
10) `unstake(id)` - Unstake all the tokens from a stake id and get all the associated rewards. Can be called only by the staker of that stake id.
