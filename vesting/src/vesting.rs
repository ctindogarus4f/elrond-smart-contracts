#![no_std]

elrond_wasm::imports!();

mod types;

use types::BeneficiaryInfo;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[elrond_wasm::derive::contract]
pub trait VestingContract {
    #[init]
    fn init(&self) {}

    // storage

    #[view(getBeneficiaryInfo)]
    #[storage_mapper("beneficiaryInfo")]
    fn beneficiary_info(
        &self,
        beneficiary: &ManagedAddress,
    ) -> SingleValueMapper<BeneficiaryInfo<Self::Api>>;
}
