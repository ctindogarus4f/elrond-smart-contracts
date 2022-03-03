#![no_std]

elrond_wasm::imports!();

mod types;

use types::BeneficiaryInfo;

/// A vesting contract that can release its token balance gradually like a typical vesting scheme.
#[elrond_wasm::derive::contract]
pub trait VestingContract {
    #[init]
    fn init(&self, token_identifier: TokenIdentifier) {
        self.token_identifier().set_if_empty(&token_identifier);
    }

    // endpoints

    #[endpoint(addBeneficiary)]
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
            tokens_claimed: BigUint::zero(),
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

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &available_tokens,
            b"successful claim",
        );

        self.beneficiary_info(&caller)
            .update(|beneficiary| beneficiary.tokens_claimed += &available_tokens);
    }

    // view functions

    #[view(getAvailableTokens)]
    fn get_available_tokens(&self) -> BigUint {
        let caller = self.blockchain().get_caller();
        let claimed_tokens = self.beneficiary_info(&caller).get().tokens_claimed;
        let vested_tokens = self.get_vested_tokens(&caller);

        vested_tokens - claimed_tokens
    }

    // private functions

    fn get_vested_tokens(&self, beneficiary: &ManagedAddress) -> BigUint {
        let beneficiary_info = self.beneficiary_info(beneficiary).get();
        let allocated_tokens = beneficiary_info.tokens_allocated;
        let first_release = beneficiary_info.start + beneficiary_info.release_cliff;
        let no_of_releases_after_cliff =
            self.get_no_of_releases_after_cliff(beneficiary_info.release_percentage);
        let last_release =
            first_release + beneficiary_info.release_duration * no_of_releases_after_cliff as u64;

        let current_timestamp = self.blockchain().get_block_timestamp();

        if current_timestamp < first_release {
            return BigUint::zero();
        } else if current_timestamp >= last_release {
            return allocated_tokens.clone();
        } else {
            let no_of_releases_until_now =
                1 + (current_timestamp - first_release) / beneficiary_info.release_duration;
            return allocated_tokens
                * beneficiary_info.release_percentage
                * no_of_releases_until_now;
        }
    }

    fn get_no_of_releases_after_cliff(&self, num: u64) -> u64 {
        if 100 % num == 0 {
            return 100 / num - 1;
        }
        100 / num
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
