## Installation

Clone this repository:  
`git clone https://github.com/S4F-IT/elrond-smart-contracts`

Install dependencies:  
`cd vesting && erdpy contract build`

## Tests

For UNIT tests, run:  
`erdpy contract test`

## Deployment

1. Access https://testnet-wallet.elrond.com/issue-token and issue your own Fungible ESDT token.
2. Edit `erdpy.json` and make sure that:
- `arguments` parameter points to the token that you've created in step 1
- `pem` parameter points to your pem file

To deploy the vesting contract, run:  
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
4. You define some groups via `addGroup` calls (ex: team group, advisor group, seed group)
5. You define some beneficiaries via `addBeneficiary` calls.

## Implementation details

All the groups are saved as a key/value map inside the storage. The key represents the group name (team, advisor, seed etc.) and the value represents a structure with details about that group (release_cliff, release_frequency, release_percentage etc.).  
Example:
```
team -> {current_allocation: 100, max_allocation: 1000, release_cliff: 31104000, release_frequency: 10368000, release_percentage: 15}
```

All the beneficiaries are saved as a key/value map inside the storage. The key represents the beneficiary wallet address and the value represents an array of ids. Each id represents a different group allocation for that beneficiary (ex: if the beneficiary is part of the team, but also invested in the seed round, he will get 2 packages: one for the `team` group and one for the `seed` group. For fetching more information about the id, each id is stored in a different key/value map. The key represents the id and the value represents a structure with information about that specific id).
Example:
```
erd1z72y3rwuz2qga43m7evtexgy0vkvv8g9uukvz4emfngxara4n94qs0ruxp -> [5, 6]
5 -> {can_be_revoked: true, is_revoked: false, group_name: team, start: 1648587600, tokens_allocated: 5000, tokens_claimed: 250}
6 -> {can_be_revoked: false, is_revoked: false, group_name: seed, start: 1648587600, tokens_allocated: 2000, tokens_claimed: 450}
```

## Available methods that can be called

***Read methods***
1) `getTokensAvailable(id)`: Get the total amount of tokens that are available to be claimed for a specific id.
2) `getTotalTokensAllocated()`: Get the total amount of tokens that were allocated to all beneficiaries.
3) `getTotalTokensClaimed()`: Get the total amount of tokens that were claimed by all beneficiaries.
4) `getTokenIdentifier()`: The token that is received by the beneficiaries.
5) `getBeneficiaryCounter()`: The total number of ids.
6) `getBeneficiaryInfo(id)`: The details about a specific id.
7) `getBeneficiaryIds(beneficiary)`: An array with ids for a specific beneficiary.
8) `getGroupInfo(group_name)`: Get all the information about a specific group.

***Write methods***
1) `claimTokensUnallocated()`: Some amount of tokens may remain unallocated. This method lets the owner of the SC to retrieve all the unallocated tokens from the SC to his personal wallet.
2) `addGroup(group_name, max_allocation, release_cliff, release_frequency, release_percentage)`: Define a new group inside the SC. Can be called only by the owner of the SC.
3) `addBeneficiary(addr, can_be_revoked, group_name, start, tokens_allocated)`: Define a new beneficiary inside the SC. Can be called only by the owner.
4) `removeBeneficiary(addr, id)`: Remove a specific id for a specific beneficiary from the sc. Why we need this? If a team member leaves the team after a few months we need to remove his benefits (note that this will remove only the id representing his `team` package. However if the team member also invested in the seed/private round, the id for that specific package will not be removed). Can be called only by the owner.
5) `claim(id)`: Claim any available tokens from a specific package id. Can be called only by the beneficiary.
