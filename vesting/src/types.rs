use elrond_wasm::{api::ManagedTypeApi, types::BigUint};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct BeneficiaryInfo<M: ManagedTypeApi> {
    pub group_type: GroupType,

    pub start: u64, // start of the vesting scheme
    pub tokens_allocated: BigUint<M>,
    pub tokens_claimed: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct GroupInfo {
    pub release_cliff: u64,
    pub release_percentage: u64,
    pub release_duration: u64,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum GroupType {
    Developer,
    Investor,
}
