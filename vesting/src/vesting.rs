#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();

mod types;

use types::*;

/// A vesting contract that can release its token balance gradually like a typical vesting scheme.
#[elrond_wasm::derive::contract]
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
        self.total_tokens_prestaked().set_if_empty(&BigUint::zero());
        self.prestake_limit().set_if_empty(&BigUint::zero());
        self.beneficiary_counter().set_if_empty(&0);
    }

    // endpoints

    #[only_owner]
    #[endpoint(setPrestakeLimit)]
    fn set_prestake_limit(&self, new_limit: BigUint) {
        require!(
            new_limit > self.total_tokens_prestaked().get(),
            "the new limit must cover the existing prestake"
        );
        self.prestake_limit().set(&new_limit);
    }

    #[only_owner]
    #[endpoint(claimTokensUnallocated)]
    fn claim_tokens_unallocated(&self) {
        let caller = self.blockchain().get_caller();
        let tokens_unallocated = self.get_tokens_unallocated();
        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &tokens_unallocated,
            b"successful claim by the owner",
        );
    }

    #[only_owner]
    #[endpoint(addGroup)]
    fn add_group(
        &self,
        group_name: ManagedBuffer,
        max_allocation: BigUint,
        release_cliff: u64,
        release_frequency: u64,
        release_percentage: u8,
    ) {
        require!(
            self.group_info(&group_name).is_empty(),
            "group has already been defined"
        );
        require!(
            release_percentage > 0 && release_percentage <= 100,
            "release percentage should be between (0, 100]"
        );

        let group_info = GroupInfo {
            current_allocation: BigUint::zero(),
            max_allocation,
            release_cliff,
            release_percentage,
            release_frequency,
        };

        self.group_info(&group_name).set(&group_info);
        self.add_group_event(&group_name, &group_info);
    }

    #[only_owner]
    #[endpoint(addBeneficiary)]
    fn add_beneficiary(
        &self,
        addr: ManagedAddress,
        can_be_revoked: bool,
        group_name: ManagedBuffer,
        start: u64,
        tokens_allocated: BigUint,
    ) {
        let mut beneficiary_ids;
        if self.beneficiary_ids(&addr).is_empty() {
            beneficiary_ids = ManagedVec::new();
        } else {
            beneficiary_ids = self.beneficiary_ids(&addr).get();
        }
        for beneficiary_id in beneficiary_ids.iter() {
            let info = self.beneficiary_info(beneficiary_id).get();
            require!(
                info.group_name != group_name,
                "beneficiary is already defined for this group"
            );
        }

        require!(
            !self.group_info(&group_name).is_empty(),
            "specified group is not set up"
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

        let group_info = self.group_info(&group_name).get();
        let new_group_current_allocation = group_info.current_allocation + &tokens_allocated;
        require!(
            new_group_current_allocation <= group_info.max_allocation,
            "group exceeds max allocation"
        );

        self.group_info(&group_name).update(|group| {
            group.current_allocation = new_group_current_allocation;
        });
        self.total_tokens_allocated()
            .set(&new_total_tokens_allocated);

        let beneficiary_info = BeneficiaryInfo {
            can_be_revoked,
            is_revoked: false,
            group_name,
            start,
            tokens_allocated,
            tokens_claimed: BigUint::zero(),
            tokens_prestaked: BigUint::zero(),
        };
        let beneficiary_counter = self.get_and_increase_beneficiary_counter();
        beneficiary_ids.push(beneficiary_counter);
        self.beneficiary_ids(&addr).set(beneficiary_ids);
        self.beneficiary_info(beneficiary_counter)
            .set(&beneficiary_info);
        self.add_beneficiary_event(&addr, &beneficiary_info)
    }

    #[only_owner]
    #[endpoint(removeBeneficiary)]
    fn remove_beneficiary(&self, addr: ManagedAddress, id: u64) {
        require!(
            !self.beneficiary_ids(&addr).is_empty(),
            "beneficiary does not exist"
        );
        let beneficiary_ids = self.beneficiary_ids(&addr).get();
        require!(
            beneficiary_ids.contains(&id),
            "id is not defined for the beneficiary"
        );

        let beneficiary_info = self.beneficiary_info(id).get();
        require!(!beneficiary_info.is_revoked, "beneficiary already removed");
        require!(
            beneficiary_info.can_be_revoked,
            "beneficiary cannot be removed"
        );

        let tokens_available = self.get_tokens_available(id);
        let new_tokens_allocated = &beneficiary_info.tokens_claimed + &tokens_available;

        self.total_tokens_allocated().update(|tokens| {
            *tokens = tokens.clone() - &beneficiary_info.tokens_allocated + &new_tokens_allocated
        });

        self.group_info(&beneficiary_info.group_name)
            .update(|group| {
                group.current_allocation = &group.current_allocation
                    - &beneficiary_info.tokens_allocated
                    + &new_tokens_allocated;
            });

        let new_tokens_prestaked = if beneficiary_info.tokens_prestaked <= new_tokens_allocated {
            beneficiary_info.tokens_prestaked
        } else {
            new_tokens_allocated.clone()
        };
        self.beneficiary_info(id).update(|beneficiary| {
            beneficiary.is_revoked = true;
            beneficiary.tokens_allocated = new_tokens_allocated;
            beneficiary.tokens_prestaked = new_tokens_prestaked;
        });
        self.remove_beneficiary_event(&addr);
    }

    #[endpoint]
    fn prestake(&self, id: u64, amount: BigUint) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.beneficiary_ids(&caller).is_empty(),
            "beneficiary does not exist"
        );

        let beneficiary_ids = self.beneficiary_ids(&caller).get();
        require!(
            beneficiary_ids.contains(&id),
            "id is not defined for the beneficiary"
        );

        let beneficiary_info = self.beneficiary_info(id).get();
        let group_info = self.group_info(&beneficiary_info.group_name).get();

        require!(!beneficiary_info.is_revoked, "beneficiary has been removed");
        require!(
            beneficiary_info.tokens_claimed == BigUint::zero(),
            "prestake is available only for those who did not claim their tokens"
        );

        let new_tokens_prestaked = &beneficiary_info.tokens_prestaked + &amount;
        let tokens_first_release =
            &beneficiary_info.tokens_allocated * group_info.release_percentage as u64 / 100u64;
        require!(
            new_tokens_prestaked > tokens_first_release,
            "prestaked amount must be higher than the first release"
        );
        require!(
            new_tokens_prestaked <= beneficiary_info.tokens_allocated,
            "prestaked amount exceeds the allocated amount"
        );
        require!(
            self.total_tokens_prestaked().get() + amount <= self.prestake_limit().get(),
            "the prestake limit has been reached"
        );

        self.beneficiary_info(id)
            .update(|beneficiary| beneficiary.tokens_prestaked = new_tokens_prestaked);
    }

    #[endpoint]
    fn claim(&self, id: u64) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.beneficiary_ids(&caller).is_empty(),
            "beneficiary does not exist"
        );

        let beneficiary_ids = self.beneficiary_ids(&caller).get();
        require!(
            beneficiary_ids.contains(&id),
            "id is not defined for the beneficiary"
        );

        let tokens_available = self.get_tokens_available(id);
        require!(
            tokens_available > 0,
            "no tokens are available to be claimed"
        );

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &tokens_available,
            b"successful claim",
        );

        self.total_tokens_claimed()
            .update(|tokens| *tokens += &tokens_available);
        self.beneficiary_info(id)
            .update(|beneficiary| beneficiary.tokens_claimed += &tokens_available);
        self.claim_event(&caller, &tokens_available);
    }

    // view functions

    #[view(getTokensAvailable)]
    fn get_tokens_available(&self, id: u64) -> BigUint {
        require!(
            !self.beneficiary_info(id).is_empty(),
            "beneficiary does not exist"
        );

        let beneficiary_info = self.beneficiary_info(id).get();
        let tokens_claimed = beneficiary_info.tokens_claimed;
        let tokens_prestaked = beneficiary_info.tokens_prestaked;
        let tokens_vested = self.get_tokens_vested(id);

        require!(
            tokens_vested >= tokens_prestaked,
            "prestaked amount is not vested yet"
        );

        let prestake_rewards = tokens_prestaked / 2 as u64; // APR of 50%
        tokens_vested + prestake_rewards - tokens_claimed
    }

    #[view(getTokensUnallocated)]
    fn get_tokens_unallocated(&self) -> BigUint {
        let total_rewards_allocated = self.total_tokens_prestaked().get() / 2 as u64; // APR of 50%
        let total_tokens_claimable = self.total_tokens_allocated().get() + total_rewards_allocated
            - self.total_tokens_claimed().get();
        let contract_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            contract_balance > total_tokens_claimable,
            "all the tokens inside the sc are allocated"
        );

        contract_balance - total_tokens_claimable
    }

    #[view(getAllBeneficiaryDeals)]
    fn get_all_beneficiary_deals(
        &self,
        beneficiary: ManagedAddress,
    ) -> MultiValueManagedVec<BeneficiaryInfo<Self::Api>> {
        require!(
            !self.beneficiary_ids(&beneficiary).is_empty(),
            "beneficiary does not exist"
        );

        let beneficiary_ids = self.beneficiary_ids(&beneficiary).get();
        let mut beneficiary_deals = MultiValueManagedVec::new();
        for beneficiary_id in beneficiary_ids.iter() {
            beneficiary_deals.push(self.beneficiary_info(beneficiary_id).get());
        }

        beneficiary_deals
    }

    // private functions

    fn get_tokens_vested(&self, id: u64) -> BigUint {
        let beneficiary_info = self.beneficiary_info(id).get();
        let group_info = self.group_info(&beneficiary_info.group_name).get();

        let tokens_allocated = beneficiary_info.tokens_allocated;
        let no_of_releases_after_cliff =
            self.get_no_of_releases_after_cliff(group_info.release_percentage);
        let first_release = beneficiary_info.start + group_info.release_cliff;
        let last_release =
            first_release + group_info.release_frequency * no_of_releases_after_cliff as u64;

        let current_timestamp = self.blockchain().get_block_timestamp();
        if current_timestamp < first_release {
            return BigUint::zero();
        } else if current_timestamp >= last_release || beneficiary_info.is_revoked {
            return tokens_allocated.clone();
        } else {
            let no_of_releases_until_now =
                1 + (current_timestamp - first_release) / group_info.release_frequency;
            return tokens_allocated
                * group_info.release_percentage as u64
                * no_of_releases_until_now
                / 100u64;
        }
    }

    fn get_no_of_releases_after_cliff(&self, release_percentage: u8) -> u8 {
        if 100 % release_percentage == 0 {
            return 100 / release_percentage - 1;
        }
        100 / release_percentage
    }

    fn get_and_increase_beneficiary_counter(&self) -> u64 {
        let id = self.beneficiary_counter().get();
        self.beneficiary_counter().set(&(id + 1));
        id
    }

    // events

    #[event("claim")]
    fn claim_event(&self, #[indexed] to: &ManagedAddress, #[indexed] amount: &BigUint);

    #[event("add_beneficiary")]
    fn add_beneficiary_event(
        &self,
        #[indexed] addr: &ManagedAddress,
        #[indexed] beneficiary_info: &BeneficiaryInfo<Self::Api>,
    );

    #[event("remove_beneficiary")]
    fn remove_beneficiary_event(&self, #[indexed] addr: &ManagedAddress);

    #[event("add_group")]
    fn add_group_event(
        &self,
        #[indexed] group_name: &ManagedBuffer,
        #[indexed] group_info: &GroupInfo<Self::Api>,
    );

    // storage

    #[view(getTotalTokensAllocated)]
    #[storage_mapper("totalTokensAllocated")]
    fn total_tokens_allocated(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalTokensClaimed)]
    #[storage_mapper("totalTokensClaimed")]
    fn total_tokens_claimed(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalTokensPrestaked)]
    #[storage_mapper("totalTokensPrestaked")]
    fn total_tokens_prestaked(&self) -> SingleValueMapper<BigUint>;

    #[view(getPrestakeLimit)]
    #[storage_mapper("prestakeLimit")]
    fn prestake_limit(&self) -> SingleValueMapper<BigUint>;

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getBeneficiaryCounter)]
    #[storage_mapper("beneficiaryCounter")]
    fn beneficiary_counter(&self) -> SingleValueMapper<u64>;

    #[view(getBeneficiaryInfo)]
    #[storage_mapper("beneficiaryInfo")]
    fn beneficiary_info(&self, id: u64) -> SingleValueMapper<BeneficiaryInfo<Self::Api>>;

    #[view(getBeneficiaryIds)]
    #[storage_mapper("beneficiaryIds")]
    fn beneficiary_ids(&self, beneficiary: &ManagedAddress) -> SingleValueMapper<ManagedVec<u64>>;

    #[view(getGroupInfo)]
    #[storage_mapper("groupInfo")]
    fn group_info(&self, group_name: &ManagedBuffer) -> SingleValueMapper<GroupInfo<Self::Api>>;
}
