use elrond_wasm::{api::ManagedTypeApi, types::BigUint};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct BeneficiaryInfo<M: ManagedTypeApi> {
    pub can_be_revoked: bool, // true if the beneficiary can be removed from the vesting scheme. false otherwise
    pub is_revoked: bool, // true if the beneficiary was removed from the vesting scheme. false otherwise
    pub group_type: GroupType, // group type that the beneficiary falls into (ex: Team)
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

// Discriminants are not explicit in the Rust code, they get generated automatically. Discriminats start from 0.
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum GroupType {
    Advisor,         // we will have multiple beneficiaries for this type - id 0
    IdoInvestor,     // we will have multiple beneficiaries for this type - id 1
    Marketing,       // we will have a single beneficiary for this type - id 2
    PrivateInvestor, // we will have multiple beneficiaries for this type - id 3
    SeedInvestor,    // we will have multiple beneficiaries for this type - id 4
    Staking,         // we will have a single beneficiary for this type - id 5
    Strategy,        // we will have a single beneficiary for this type - id 6
    Team,            // we will have multiple beneficiaries for this type - id 7
    Treasury,        // we will have a single beneficiary for this type - id 8
}
