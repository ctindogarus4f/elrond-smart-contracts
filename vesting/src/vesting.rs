#![no_std]

elrond_wasm::imports!();

mod types;

use types::BeneficiaryInfo;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[elrond_wasm::derive::contract]
pub trait VestingContract {
    #[init]
    fn init(&self, token_identifier: TokenIdentifier) {
        self.token_identifier().set_if_empty(&token_identifier);
    }

    // endpoints

    #[endpoint]
    fn add_beneficiary(
        &self,
        addr: ManagedAddress,
        release_cliff: u64,
        release_percentage: u64,
        release_duration: u64,
        tokens_allocated: BigUint,
    ) {
        let beneficiary_info = BeneficiaryInfo {
            start: self.blockchain().get_block_timestamp(),
            release_cliff,
            release_percentage,
            release_duration,
            tokens_allocated,
            tokens_released: BigUint::zero(),
        };

        self.beneficiary_info(&addr).set(&beneficiary_info);
    }

    #[endpoint]
    fn claim(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.beneficiary_info(&caller).is_empty(),
            "non-existent beneficiary"
        );

        let available_tokens = self.get_available_tokens();
        require!(
            available_tokens > 0,
            "no tokens are available to be claimed"
        );

        self.beneficiary_info(&caller)
            .update(|beneficiary| beneficiary.tokens_released += &available_tokens);

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &available_tokens,
            b"successful claim",
        );
    }

    // view functions

    fn get_available_tokens(&self) -> BigUint {
        // TODO: implement this method
        BigUint::zero()
    }

    // storage

    #[view(getBeneficiaryInfo)]
    #[storage_mapper("beneficiaryInfo")]
    fn beneficiary_info(
        &self,
        beneficiary: &ManagedAddress,
    ) -> SingleValueMapper<BeneficiaryInfo<Self::Api>>;

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;
}
