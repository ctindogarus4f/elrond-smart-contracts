use elrond_wasm::{api::ManagedTypeApi, types::BigUint, types::ManagedBuffer};

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct StakerInfo<M: ManagedTypeApi> {
    pub last_claim: u64,
    pub package_name: ManagedBuffer<M>,
    pub start: u64,
    pub tokens_staked: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub struct PackageInfo {
    pub apr_percentage: u8,
    pub duration: u64,
    pub daily_rewards: bool,
    pub penalty_type: PenaltyType,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum PenaltyType {
    FeePercentage { fee: u8 },
    DaysUntilUnlocked { days: u8 },
}
