#![no_std]

elrond_wasm::imports!();

mod types;

use types::*;

/// A vesting contract that can release its token balance gradually like a typical vesting scheme.
#[elrond_wasm::derive::contract]
pub trait VestingContract {
    #[init]
    fn init(&self, token_identifier: TokenIdentifier, multisig_address: ManagedAddress) {
        self.token_identifier().set_if_empty(&token_identifier);
        self.multisig_address().set_if_empty(&multisig_address);
    }

    // endpoints

    #[endpoint(changeMultisigAddress)]
    fn change_multisig_address(&self, new_multisig_address: ManagedAddress) {
        self.assert_multisig_wallet();
        self.multisig_address().set(&new_multisig_address);
    }

    #[endpoint(addGroup)]
    fn add_group(
        &self,
        group_type: GroupType,
        release_cliff: u64,
        release_frequency: u64,
        release_percentage: u8,
    ) {
        self.assert_multisig_wallet();

        require!(
            self.group_info(&group_type).is_empty(),
            "group has already been defined",
        );

        let group_info = GroupInfo {
            release_cliff,
            release_percentage,
            release_frequency,
        };

        self.group_info(&group_type).set_if_empty(&group_info);
        self.add_group_event(&group_type, &group_info);
    }

    #[endpoint(addBeneficiary)]
    fn add_beneficiary(
        &self,
        addr: ManagedAddress,
        can_be_revoked: bool,
        group_type: GroupType,
        start: u64,
        tokens_allocated: BigUint,
    ) {
        self.assert_multisig_wallet();

        require!(
            self.beneficiary_info(&addr).is_empty(),
            "beneficiary has already been added",
        );

        require!(
            !self.group_info(&group_type).is_empty(),
            "specified group is not set up",
        );

        let beneficiary_info = BeneficiaryInfo {
            can_be_revoked,
            is_revoked: false,
            group_type,
            start,
            tokens_allocated,
            tokens_claimed: BigUint::zero(),
        };

        self.beneficiary_info(&addr).set(&beneficiary_info);
        self.add_beneficiary_event(&addr, &beneficiary_info)
    }

    #[endpoint(removeBeneficiary)]
    fn remove_beneficiary(&self, addr: ManagedAddress) {
        self.assert_multisig_wallet();

        require!(
            !self.beneficiary_info(&addr).is_empty(),
            "beneficiary does not exist",
        );

        let beneficiary_info = self.beneficiary_info(&addr).get();

        require!(!beneficiary_info.is_revoked, "beneficiary already removed",);
        require!(
            beneficiary_info.can_be_revoked,
            "beneficiary cannot be removed",
        );

        let tokens_available = self.get_tokens_available(addr.clone());
        let new_tokens_allocated = beneficiary_info.tokens_claimed + tokens_available;

        self.beneficiary_info(&addr).update(|beneficiary| {
            beneficiary.is_revoked = true;
            beneficiary.tokens_allocated = new_tokens_allocated;
        });
        self.remove_beneficiary_event(&addr);
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

        let esdt_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            esdt_balance >= tokens_available,
            "not enough tokens in vesting contract"
        );

        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &tokens_available,
            b"successful claim",
        );

        self.beneficiary_info(&caller)
            .update(|beneficiary| beneficiary.tokens_claimed += &tokens_available);
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
        let tokens_vested = self.get_tokens_vested(&addr);

        tokens_vested - tokens_claimed
    }

    // private functions

    fn assert_multisig_wallet(&self) {
        let multisig_address = self.multisig_address().get(); // set in constructor
        require!(
            self.blockchain().get_caller() == multisig_address,
            "caller not authorized",
        );
    }

    fn get_tokens_vested(&self, addr: &ManagedAddress) -> BigUint {
        let beneficiary_info = self.beneficiary_info(addr).get();
        let group_info = self.group_info(&beneficiary_info.group_type).get(); // checked when set beneficiaryInfo

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
        require!(
            release_percentage > 0 && release_percentage <= 100,
            "release percentage should be between (0, 100]"
        );

        if 100 % release_percentage == 0 {
            return 100 / release_percentage - 1;
        }
        100 / release_percentage
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
    fn add_group_event(&self, #[indexed] group_type: &GroupType, #[indexed] group_info: &GroupInfo);

    // storage

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getMultisigAddress)]
    #[storage_mapper("multisigAddress")]
    fn multisig_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getBeneficiaryInfo)]
    #[storage_mapper("beneficiaryInfo")]
    fn beneficiary_info(
        &self,
        beneficiary: &ManagedAddress,
    ) -> SingleValueMapper<BeneficiaryInfo<Self::Api>>;

    #[view(getGroupInfo)]
    #[storage_mapper("groupInfo")]
    fn group_info(&self, group_type: &GroupType) -> SingleValueMapper<GroupInfo>;
}
