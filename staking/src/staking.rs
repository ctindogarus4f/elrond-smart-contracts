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

        let staker_info = StakerInfo {
            package_name,
            start: self.blockchain().get_block_timestamp(),
            tokens_staked: payment_amount,
        };

        self.staker_info(&caller).set(&staker_info);
    }

    #[endpoint]
    fn unstake(&self) {}

    // private functions

    fn assert_multisig_wallet(&self) {
        let multisig_address = self.multisig_address().get();
        require!(
            self.blockchain().get_caller() == multisig_address,
            "caller not authorized",
        );
    }

    // storage

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
