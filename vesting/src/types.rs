use multiversx_sc::{api::ManagedTypeApi, types::BigUint, types::ManagedBuffer};

multiversx_sc::derive_imports!();

#[derive(ManagedVecItem, NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct BeneficiaryInfo<M: ManagedTypeApi> {
    pub can_be_revoked: bool, // true if the beneficiary can be removed from the vesting scheme. false otherwise
    pub is_revoked: bool, // true if the beneficiary was removed from the vesting scheme. false otherwise
    pub group_name: ManagedBuffer<M>, // group name that the beneficiary falls into (ex: team)
    pub start: u64,       // start of the vesting scheme as a unix timestamp
    pub tokens_allocated: BigUint<M>, // amount of tokens allocated to this beneficiary
    pub tokens_claimed: BigUint<M>, // amount of tokens claimed by this beneficiary
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct GroupInfo<M: ManagedTypeApi> {
    pub current_allocation: BigUint<M>, // tokens allocated to this group so far
    pub max_allocation: BigUint<M>,     // max amount of tokens that can be allocated to this group
    pub release_cliff: u64,             // moment when the first release takes place
    pub release_frequency: u64,         // frequency of each release
    pub release_percentage: u8,         // percentage that gets released from the allocated tokens
}
