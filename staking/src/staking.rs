#![no_std]

elrond_wasm::imports!();

mod types;

use types::*;

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

        let caller = self.blockchain().get_caller();
        self.multisig_address().set_if_empty(&caller);
        self.total_tokens_allocated().set_if_empty(&BigUint::zero());
    }

    // endpoints

    #[endpoint(addPackage)]
    fn add_package(&self, package_name: ManagedBuffer, apr_percentage: u8, locking_period: u64) {
        self.assert_multisig_wallet();

        require!(
            self.package_info(&package_name).is_empty(),
            "package has already been defined",
        );
        require!(
            apr_percentage > 0 && apr_percentage <= 100,
            "apr percentage should be between (0, 100]"
        );

        let package_info = PackageInfo {
            apr_percentage,
            locking_period,
        };

        self.package_info(&package_name).set(&package_info);
    }

    #[payable("*")]
    #[endpoint]
    fn stake(&self, package_name: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        require!(
            self.staker_info(&caller).is_empty(),
            "staker has already been added",
        );

        require!(
            !self.package_info(&package_name).is_empty(),
            "specified package is not set up",
        );

        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(
            payment_token == self.token_identifier().get(),
            "invalid staked token"
        );

        let package_info = self.package_info(&package_name).get();
        let unstake_amount =
            self.compute_unstake_amount(&payment_amount, package_info.apr_percentage);
        self.total_tokens_allocated()
            .update(|tokens| *tokens += unstake_amount);

        let esdt_balance = self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            &self.token_identifier().get(),
            0,
        );
        require!(
            esdt_balance >= self.total_tokens_allocated().get(),
            "not enough tokens in staking contract"
        );

        let staker_info = StakerInfo {
            package_name,
            start: self.blockchain().get_block_timestamp(),
            tokens_staked: payment_amount,
        };

        self.staker_info(&caller).set(&staker_info);
    }

    #[endpoint]
    fn unstake(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.staker_info(&caller).is_empty(),
            "staker does not exist",
        );

        let staker_info = self.staker_info(&caller).get();
        let package_info = self.package_info(&staker_info.package_name).get();

        require!(
            staker_info.start + package_info.locking_period
                >= self.blockchain().get_block_timestamp(),
            "cannot unstake sooner than the locking period"
        );

        let unstake_amount =
            self.compute_unstake_amount(&staker_info.tokens_staked, package_info.apr_percentage);
        self.send().direct(
            &caller,
            &self.token_identifier().get(),
            0,
            &unstake_amount,
            b"successful unstake",
        );

        self.staker_info(&caller).clear();
    }

    // private functions

    fn assert_multisig_wallet(&self) {
        let multisig_address = self.multisig_address().get();
        require!(
            self.blockchain().get_caller() == multisig_address,
            "caller not authorized",
        );
    }

    fn compute_unstake_amount(&self, staked_amount: &BigUint, apr_percentage: u8) -> BigUint {
        staked_amount * (100 + apr_percentage) as u64 / 100u64
    }

    // storage

    #[view(getTotalTokensAllocated)]
    #[storage_mapper("totalTokensAllocated")]
    fn total_tokens_allocated(&self) -> SingleValueMapper<BigUint>;

    #[view(getTokenIdentifier)]
    #[storage_mapper("tokenIdentifier")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getMultisigAddress)]
    #[storage_mapper("multisigAddress")]
    fn multisig_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getStakerInfo)]
    #[storage_mapper("stakerInfo")]
    fn staker_info(&self, staker: &ManagedAddress) -> SingleValueMapper<StakerInfo<Self::Api>>;

    #[view(getPackageInfo)]
    #[storage_mapper("packageInfo")]
    fn package_info(&self, package_name: &ManagedBuffer) -> SingleValueMapper<PackageInfo>;
}
