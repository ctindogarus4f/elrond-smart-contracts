#![no_std]

elrond_wasm::imports!();

/// A staking contract. Users can stake ESDT tokens and gradually receive ESDT token rewards.
#[elrond_wasm::derive::contract]
pub trait StakingContract {
    #[init]
    fn init(&self, token_identifier: TokenIdentifier) {
        require!(
            token_identifier.is_valid_esdt_identifier(),
            "invalid esdt token"
        );
        self.token_identifier().set_if_empty(&token_identifier);
    }

    // storage

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;
}
