#![no_std]

elrond_wasm::imports!();

mod types;

use types::*;

/// A vesting contract that can release its token balance gradually like a typical vesting scheme.
#[elrond_wasm::contract]
pub trait VestingContract {
    #[init]
    fn init(&self, token_identifier: TokenIdentifier) {
        require!(
            token_identifier.is_valid_esdt_identifier(),
            "invalid esdt token"
        );
        self.token_identifier().set_if_empty(&token_identifier);
        self.total_tokens_allocated().set_if_empty(&BigUint::zero());
        self.total_tokens_claimed().set_if_empty(&BigUint::zero());
        self.beneficiary_counter().set_if_empty(&0);
    }

    // endpoints

    #[only_owner]
    #[endpoint(claimTokensUnallocated)]
    fn claim_tokens_unallocated(&self) {
        let caller = self.blockchain().get_caller();
        let total_tokens_claimable =
            self.total_tokens_allocated().get() - self.total_tokens_claimed().get();
        let contract_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            contract_balance > total_tokens_claimable,
            "nothing to claim. all the tokens in the sc are allocated"
        );

        let unallocated_tokens = contract_balance - total_tokens_claimable;
        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &unallocated_tokens,
            b"successful claim by the owner",
        );
        self.claim_tokens_unallocated_event(&unallocated_tokens);
    }

    #[only_owner]
    #[endpoint(replaceWallet)]
    fn replace_wallet(&self, old_wallet: ManagedAddress, new_wallet: ManagedAddress) {
        require!(
            !self.beneficiary_info(&old_wallet).is_empty(),
            "beneficiary does not exist"
        );

        let beneficiary_info = self.beneficiary_info(&old_wallet).get();
        self.beneficiary_info(&new_wallet).set(beneficiary_info);
        self.beneficiary_info(&old_wallet).clear();

        self.replace_wallet_event(&old_wallet, &new_wallet);
    }

    #[only_owner]
    #[endpoint(addGroup)]
    fn add_group(&self, group_name: ManagedBuffer, release_frequency: u64, lock_period: u64) {
        require!(
            self.group_info(&group_name).is_empty(),
            "group has already been defined",
        );
        require!(
            lock_period % release_frequency == 0,
            "lock period must be divisible with release frequency"
        );

        let group_info = GroupInfo {
            release_frequency,
            lock_period,
        };

        self.group_info(&group_name).set(&group_info);
        self.add_group_event(&group_name, &group_info);
    }

    #[only_owner]
    #[endpoint(addMultipleBeneficiaries)]
    fn add_multiple_beneficiaries(
        &self,
        group_name: ManagedBuffer,
        start: u64,
        args: MultiValueEncoded<MultiValue2<ManagedAddress, BigUint>>,
    ) {
        require!(
            !self.group_info(&group_name).is_empty(),
            "specified group is not set up",
        );

        let mut total_tokens_allocated = self.total_tokens_allocated().get();
        for beneficiary_pair in args.into_iter() {
            let (addr, tokens_allocated) = beneficiary_pair.into_tuple();
            require!(
                self.beneficiary_info(&addr).is_empty(),
                "beneficiary is already defined for this group",
            );

            let beneficiary_info = BeneficiaryInfo {
                group_name: group_name.clone(),
                start,
                tokens_allocated: tokens_allocated.clone(),
                tokens_claimed: BigUint::zero(),
            };
            self.beneficiary_info(&addr).set(&beneficiary_info);

            total_tokens_allocated += &tokens_allocated;
        }

        self.total_tokens_allocated().set(&total_tokens_allocated);
    }

    #[only_owner]
    #[endpoint(addBeneficiary)]
    fn add_beneficiary(
        &self,
        addr: ManagedAddress,
        group_name: ManagedBuffer,
        start: u64,
        tokens_allocated: BigUint,
    ) {
        require!(
            !self.group_info(&group_name).is_empty(),
            "specified group is not set up",
        );
        require!(
            self.beneficiary_info(&addr).is_empty(),
            "beneficiary is already defined for this group",
        );

        let new_total_tokens_allocated = self.total_tokens_allocated().get() + &tokens_allocated;
        let total_tokens_claimable =
            &new_total_tokens_allocated - &self.total_tokens_claimed().get();
        let contract_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            contract_balance >= total_tokens_claimable,
            "not enough tokens in vesting contract"
        );

        self.total_tokens_allocated()
            .set(&new_total_tokens_allocated);

        let beneficiary_info = BeneficiaryInfo {
            group_name: group_name.clone(),
            start,
            tokens_allocated: tokens_allocated.clone(),
            tokens_claimed: BigUint::zero(),
        };
        self.beneficiary_info(&addr).set(&beneficiary_info);
        self.add_beneficiary_event(&addr, &group_name, start, &tokens_allocated)
    }

    #[endpoint]
    fn claim(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.beneficiary_info(&caller).is_empty(),
            "beneficiary does not exist"
        );

        let tokens_available = self.get_tokens_available(caller.clone());
        require!(
            tokens_available > 0,
            "no tokens are available to be claimed"
        );

        self.total_tokens_claimed()
            .update(|tokens| *tokens += &tokens_available);
        self.beneficiary_info(&caller)
            .update(|beneficiary| beneficiary.tokens_claimed += &tokens_available);

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &tokens_available,
            b"successful claim",
        );
        self.claim_event(&caller, &tokens_available);
    }

    // view functions

    #[view(getTokensAvailable)]
    fn get_tokens_available(&self, addr: ManagedAddress) -> BigUint {
        require!(
            !self.beneficiary_info(&addr).is_empty(),
            "beneficiary does not exist"
        );

        let tokens_claimed = self.beneficiary_info(&addr).get().tokens_claimed;
        let tokens_vested = self.get_tokens_vested(addr);

        tokens_vested - tokens_claimed
    }

    #[view(getTokensVested)]
    fn get_tokens_vested(&self, addr: ManagedAddress) -> BigUint {
        let beneficiary_info = self.beneficiary_info(&addr).get();
        let group_info = self.group_info(&beneficiary_info.group_name).get();

        let tokens_allocated = beneficiary_info.tokens_allocated;
        let total_no_of_releases = group_info.lock_period / group_info.release_frequency;
        let first_release = beneficiary_info.start + group_info.release_frequency;
        let last_release = beneficiary_info.start + group_info.lock_period;

        let current_timestamp = self.blockchain().get_block_timestamp();
        if current_timestamp < first_release {
            BigUint::zero()
        } else if current_timestamp >= last_release {
            tokens_allocated
        } else {
            let no_of_releases_until_now =
                (current_timestamp - beneficiary_info.start) / group_info.release_frequency;
            tokens_allocated * no_of_releases_until_now / total_no_of_releases
        }
    }

    // events

    #[event("claim_tokens_unallocated")]
    fn claim_tokens_unallocated_event(&self, #[indexed] amount: &BigUint);

    #[event("claim")]
    fn claim_event(&self, #[indexed] to: &ManagedAddress, #[indexed] amount: &BigUint);

    #[event("replace_wallet")]
    fn replace_wallet_event(
        &self,
        #[indexed] old_wallet: &ManagedAddress,
        #[indexed] new_wallet: &ManagedAddress,
    );

    #[event("add_beneficiary")]
    fn add_beneficiary_event(
        &self,
        #[indexed] addr: &ManagedAddress,
        #[indexed] group_name: &ManagedBuffer,
        #[indexed] start: u64,
        #[indexed] amount: &BigUint,
    );

    #[event("add_group")]
    fn add_group_event(
        &self,
        #[indexed] group_name: &ManagedBuffer,
        #[indexed] group_info: &GroupInfo,
    );

    // storage

    #[view(getTotalTokensAllocated)]
    #[storage_mapper("totalTokensAllocated")]
    fn total_tokens_allocated(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalTokensClaimed)]
    #[storage_mapper("totalTokensClaimed")]
    fn total_tokens_claimed(&self) -> SingleValueMapper<BigUint>;

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getBeneficiaryCounter)]
    #[storage_mapper("beneficiaryCounter")]
    fn beneficiary_counter(&self) -> SingleValueMapper<u64>;

    #[view(getBeneficiaryInfo)]
    #[storage_mapper("beneficiaryInfo")]
    fn beneficiary_info(
        &self,
        beneficiary: &ManagedAddress,
    ) -> SingleValueMapper<BeneficiaryInfo<Self::Api>>;

    #[view(getGroupInfo)]
    #[storage_mapper("groupInfo")]
    fn group_info(&self, group_name: &ManagedBuffer) -> SingleValueMapper<GroupInfo>;
}
