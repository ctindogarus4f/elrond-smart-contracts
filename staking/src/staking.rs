#![no_std]

multiversx_sc::imports!();

/// A staking contract. Users can stake ESDT tokens and gradually receive ESDT token rewards.
#[multiversx_sc::contract]
pub trait StakingContract {
    #[init]
    fn init(&self, token_identifier: TokenIdentifier) {
        self.token_identifier().set_if_empty(&token_identifier);
    }

    #[payable("*")]
    #[endpoint(createNewStake)]
    fn create_new_stake(&self) {

    }

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;
}
