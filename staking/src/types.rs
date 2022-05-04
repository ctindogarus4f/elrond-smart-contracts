use elrond_wasm::{api::ManagedTypeApi, types::BigUint, types::ManagedBuffer};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct StakerInfo<M: ManagedTypeApi> {
    pub package_name: ManagedBuffer<M>,
    pub stake_timestamp: u64,
    pub tokens_staked: BigUint<M>,
    pub last_claim_of_rewards: u64,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct PackageInfo {
    pub valid_until_timestamp: u64,
    pub apr_percentage: u8,
    pub rewards_frequency: u64, // in days
    pub penalty_type: PenaltyType,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum PenaltyType {
    FeePercentage { fee: u8 },
    DaysUntilUnlocked { days: u8 },
}
