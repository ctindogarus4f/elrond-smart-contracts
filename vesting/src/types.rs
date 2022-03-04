use elrond_wasm::{api::ManagedTypeApi, types::BigUint};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct BeneficiaryInfo<M: ManagedTypeApi> {
    pub can_be_revoked: bool,
    pub is_revoked: bool,
    pub group_type: GroupType,
    pub start: u64,                   // start of the vesting scheme
    pub tokens_allocated: BigUint<M>, // amount of tokens allocated to this beneficiary
    pub tokens_claimed: BigUint<M>,   // amount of tokens claimed by this beneficiary
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct GroupInfo {
    pub release_cliff: u64,
    pub release_duration: u64,
    pub release_percentage: u8,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum GroupType {
    Advisor,
    Marketing,
    PrivateInvestor,
    SeedInvestor,
    Strategy,
    Team,
    Treasury,
}
