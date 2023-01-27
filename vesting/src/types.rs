use elrond_wasm::{api::ManagedTypeApi, types::BigUint, types::ManagedBuffer};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct BeneficiaryInfo<M: ManagedTypeApi> {
    pub group_name: ManagedBuffer<M>, // group name that the beneficiary falls into (ex: team)
    pub start: u64,                   // start of the vesting scheme as a unix timestamp
    pub tokens_allocated: BigUint<M>, // amount of tokens allocated to this beneficiary
    pub tokens_claimed: BigUint<M>,   // amount of tokens claimed by this beneficiary
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct GroupInfo {
    pub release_frequency: u64, // frequency of each release, in seconds
    pub lock_period: u64,       // seconds until fully vested
}
