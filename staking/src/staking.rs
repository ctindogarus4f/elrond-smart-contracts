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
    fn add_package(&self, package_name: ManagedBuffer, apr_percentage: u8, locking_period: u32) {
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

    #[endpoint]
    fn stake(&self) {}

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

    #[view(getPackageInfo)]
    #[storage_mapper("packageInfo")]
    fn package_info(&self, package_name: &ManagedBuffer) -> SingleValueMapper<PackageInfo>;
}
